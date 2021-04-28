"""Mission Pinball Framework Rust Media Controller (mpf-mc-rust) setup.py."""
import setuptools

with open("README.md", "r") as fh:
    long_description = fh.read()

setuptools.setup(
    name="mpf-mc-rust",
    version="0.0.1",
    author="The Mission Pinball Framework Team",
    author_email="jan-mpf@kantert.net",
    description="A media controller for Mission Pinball games, powered by Rust",
    long_description=long_description,
    long_description_content_type="text/markdown",
    url="https://github.com/missionpinball/mpf-mc-rust",
    packages=setuptools.find_packages(),
    classifiers=[
        "Programming Language :: Python :: 3",
        "License :: UNLICENSED",
        "Operating System :: OS Independent",
    ],
    install_requires=[
        'grpcio',
        'grpcio-tools',
        'protobuf',
    ],
)
