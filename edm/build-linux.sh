docker pull quay.io/pypa/manylinux1_x86_64
docker run --rm -v `pwd`:/io quay.io/pypa/manylinux1_x86_64 /io/build-wheels.sh
# ^
# #!/bin/bash
# set -ex
# curl https://sh.rustup.rs -sSf | sh -s -- -y
# export PATH="$HOME/.cargo/bin:$PATH"
# cd /io
# for PYBIN in /opt/python/{cp35-cp35m,cp36-cp36m,cp37-cp37m}/bin; do
#     export PYTHON_SYS_EXECUTABLE="$PYBIN/python"
#     "${PYBIN}/pip" install -U setuptools wheel setuptools-rust
#     "${PYBIN}/python" setup.py bdist_wheel
# done
# for whl in dist/*.whl; do
#     auditwheel repair "$whl" -w dist/
# done
