from setuptools import setup
from setuptools_rust import Binding, RustExtension

with open("README.md", "r") as f:
    long_description = f.read()

setup(
    name="edm",
    version="0.0",
    author="David Golembiowski, Nik Sidnev",
    maintainer="David Golembiowski, Nik Sidnev",
    author_email="david[at]dgolembiowski[dot]com , sidnev.nick[at]gmail[dot]com",
    maintainer_email="david[at]dgolembiowski[dot]com, sidnev.nick[at]gmail[dot]com",
    keywords=["edgemorph", "edgedb", "orm", "frm", "compiler", "cli"],
    description="edm: edgemorph development manager",
    license="Apache License, Version 2.0",
    long_description=long_description,
    long_description_content_type="text/markdown"
    rust_extensions=[RustExtension("edm.edm", binding=Binding.PyO3)],
    packages=["edm"],
    zip_safe=False,
    classifiers=[
        "Development Status :: 1 - Planning",
        "Programming Language :: Python",
        "Programming Language :: Python :: 3.8",
        "Programming Language :: Python :: 3.9",
        "Programming Language :: Python :: 3 :: Only",
        "Programming Language :: Rust",
        "Operating System :: Microsoft :: Windows",
        "Operating System :: POSIX",
        "Operating System :: Unix",
        "Operating System :: MacOS :: MacOS X",
        "Framework :: Edgemorph"
    ]

)
