MPF MC Media Controller in Rust
===============================

Compile and run
---------------

* Install Rust via https://rustup.rs/
* Install gstreamer according to: https://crates.io/crates/gstreamer
* Create an `assets` folder
* Download Fire Sans from https://fonts.google.com/specimen/Fira+Sans, uzip and put `FiraSans-Regular.ttf` into the `assets` folder
* Run `cargo run` within the repository

Test using the Python Demo Client
---------------------------------

* Install grpcio and protobuf: `pip3 install grpcio grpcio-tools protobuf`
* Change to the python directory
* Modify the image/video path in `test.py` to an existing image on your disk
* Run `python3 test.py`

