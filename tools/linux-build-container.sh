#!/bin/sh

docker run \
    -it \
    --volume=$(pwd):/home/circleci/project \
    olback/rust-gtk-linux /bin/bash \
