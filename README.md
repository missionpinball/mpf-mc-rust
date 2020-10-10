MPF MC Media Controller in Rust
===============================

Compile and run
---------------

* Install Rust via https://rustup.rs/
* Install gstreamer according to: https://crates.io/crates/gstreamer
* Create an `resources` folder
* Download `https://github.com/ggez/ggez/raw/master/resources/DejaVuSerif.ttf` and put it into the `resources` folder
* Run `cargo run` within the repository

Test using the Python Demo Client
---------------------------------

* Install grpcio and protobuf: `pip3 install grpcio grpcio-tools protobuf`
* Change to the python directory
* Modify the image/video path in `test.py` to an existing image on your disk
* Run `python3 test.py`

