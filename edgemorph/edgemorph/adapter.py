import sys
from functools import wraps
from dataclasses import (
        _cmp_fn,
        dataclass,
        _DataclassParams,
        field,
        Field,
        _FIELD,
        _FIELDS,
        _FIELD_INITVAR,
        _get_field,
        _hash_action,
        _init_fn,
        MISSING,
        _PARAMS,
        _POST_INIT_NAME,
        _repr_fn,
        _set_new_attribute,
        _tuple_str,)

import inspect
import types
from typing import ( Tuple, Union, Any )

__all__ = ( "edgetype" )

def _proc(cls, is_abstract, extending, init, repr, eq, order, unsafe_hash, frozen):
    
    fields = {}

    if cls.__module__ in sys.modules:
        globals = sys.modules[cls.__module__].__dict__
    else:
        # Theoretically this can happen if someone writes
        # a custom string to cls.__module__.  In which case
        # such dataclass won't be fully introspectable
        # (w.r.t. typing.get_type_hints) but will still function
        # correctly.
        globals = {}
    setattr(cls, _PARAMS, _DataclassParams(init, repr, eq, order,
                                           unsafe_hash, frozen)) 
    any_frozen_base = False
    has_dataclass_bases = False
    for b in cls.__mro__[-1:0:-1]:
        # Only process classes that have been processed by our
        # decorator.  That is, they have a _FIELDS attribute.
        base_fields = getattr(b, _FIELDS, None)
        if base_fields:
            has_dataclass_bases = True
            for f in base_fields.values():
                fields[f.name] = f
            if getattr(b, _PARAMS).frozen:
                any_frozen_base = True

    # Annotations that are defined in this class (not in base
    # classes).  If __annotations__ isn't present, then this class
    # adds no new annotations.  We use this to compute fields that are
    # added by this class.#
    cls_annotations = cls.__dict__.get('__annotations__', {})
    cls_fields = [_get_field(cls, name, type)
                  for name, type in cls_annotations.items()]
    for f in cls_fields:
        fields[f.name] = f
        if isinstance(getattr(cls, f.name, None), Field):
            if f.default is MISSING:
                delattr(cls, f.name)
            else:
                setattr(cls, f.name, f.default)

    # Do we have any Field members that don't also have annotations?
    for name, value in cls.__dict__.items():
        if isinstance(value, Field) and not name in cls_annotations:
            raise TypeError(f'{name!r} is a field but has no type annotation')
    if has_dataclass_bases:
        if any_frozen_base and not frozen:
            raise TypeError('cannot inherit non-frozen dataclass from a '
                            'frozen one')
        if not any_frozen_base and frozen:
            raise TypeError('cannot inherit frozen dataclass from a '
                            'non-frozen one')
    setattr(cls, _FIELDS, fields)

    # Was this class defined with an explicit __hash__?  Note that if
    # __eq__ is defined in this class, then python will automatically
    # set __hash__ to None.  This is a heuristic, as it's possible
    # that such a __hash__ == None was not auto-generated, but it
    # close enough.
    class_hash = cls.__dict__.get('__hash__', MISSING)
    has_explicit_hash = not (class_hash is MISSING or
                             (class_hash is None and '__eq__' in cls.__dict__))

    # If we're generating ordering methods, we must be generating the
    # eq methods.
    if order and not eq:
        raise ValueError('eq must be true if order is true')

    if init:
        # Does this class have a post-init function?
        has_post_init = hasattr(cls, _POST_INIT_NAME)
        flds = [f for f in fields.values()
                if f._field_type in (_FIELD, _FIELD_INITVAR)]
        _set_new_attribute(cls, '__init__',
                           _init_fn(flds,
                                    frozen,
                                    has_post_init,
                                    # The name to use for the "self"
                                    # param in __init__.  Use "self"
                                    # if possible.
                                    '__dataclass_self__' if 'self' in fields
                                            else 'self',
                                    globals,
                          ))

    # Get the fields as a list, and include only real fields.  This is
    # used in all of the following methods.
    field_list = [f for f in fields.values() if f._field_type is _FIELD]

    if repr:
        flds = [f for f in field_list if f.repr]
        _set_new_attribute(cls, '__repr__', _repr_fn(flds, globals))

    if eq:
        # Create _eq__ method.  There's no need for a __ne__ method,
        # since python will call __eq__ and negate it.
        flds = [f for f in field_list if f.compare]
        self_tuple = _tuple_str('self', flds)
        other_tuple = _tuple_str('other', flds)
        _set_new_attribute(cls, '__eq__',
                           _cmp_fn('__eq__', '==',
                                   self_tuple, other_tuple,
                                   globals=globals))

    if order:
        # Create and set the ordering methods.
        flds = [f for f in field_list if f.compare]
        self_tuple = _tuple_str('self', flds)
        other_tuple = _tuple_str('other', flds)
        for name, op in [('__lt__', '<'),
                         ('__le__', '<='),
                         ('__gt__', '>'),
                         ('__ge__', '>='),
                         ]:
            if _set_new_attribute(cls, name,
                                  _cmp_fn(name, op, self_tuple, other_tuple,
                                          globals=globals)):
                raise TypeError(f'Cannot overwrite attribute {name} '
                                f'in class {cls.__name__}. Consider using '
                                'functools.total_ordering')

    if frozen:
        for fn in _frozen_get_del_attr(cls, field_list, globals):
            if _set_new_attribute(cls, fn.__name__, fn):
                raise TypeError(f'Cannot overwrite attribute {fn.__name__} '
                                f'in class {cls.__name__}')

    # Decide if/how we're going to create a hash function.
    hash_action = _hash_action[bool(unsafe_hash),
                               bool(eq),
                               bool(frozen),
                               has_explicit_hash]
    if hash_action:
        # No need to call _set_new_attribute here, since by the time
        # we're here the overwriting is unconditional.
        cls.__hash__ = hash_action(cls, field_list, globals)

    if not getattr(cls, '__doc__'):
        # Create a class doc-string.
        cls.__doc__ = (cls.__name__ +
                       str(inspect.signature(cls)).replace(' -> None', ''))
    cls.is_abstract = is_abstract
    new_annots = {}

    def cond_update(ty: object):
        nonlocal new_annots
        try:
            assert(ty.is_abstract)
            try:
                new_annots.update(ty.__annotations__)
            except AttributeError:
                # Don't care if it fails
                pass
        except AssertionError:
            # We do care if it fails
            err = ValueError(f"Cannot extend `{cls.__qualname__}` with `{ty}` because `{ty}` is not abstract. [Help: Try modifying `{ty.__qualname__}`'s decorator to `@edgetype(..., is_abstract=True)`]")
            raise(err)
    
    def update_cls():
        nonlocal new_annots
        nonlocal cls
        cls.__annotations__ = new_annots
    
    # Performance hit to ensure well-orderedness
    if not isinstance(extending, tuple):
        extending = (extending,)

    if len(extending) > 1:
        for abs_edgetype in extending:
            cond_update(abs_edgetype) 
        update_cls()
    elif len(extending) == 1:
        cond_update(extending[0])
        update_cls()
    else:
        pass

    cls.extending = extending
    return cls

def edgetype(cls=None, /, *, 
        is_abstract: bool = False,
        extending: Tuple[Union[Any, None]] = (),
        init: bool = True,
        repr: bool = True,
        eq:   bool = True,
        order:bool = False,
        unsafe_hash: bool = False,
        frozen: bool = False):
    def wrap(cls):
        return _proc(cls, is_abstract, extending, init, repr, eq, order, unsafe_hash, frozen)

    # Check for `extending`, `is_abstract`, etc.
    if cls is None:
        # We're called with parens.
        return wrap

    # No arguments supplied to `@edgetype`
    return wrap(cls)

