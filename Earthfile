VERSION 0.7

build:
    BUILD ./firmware/+build
    BUILD ./frontend/+build
    BUILD ./backend/+build

docker:
    ARG VERSION
    BUILD --VERSION=$VERSION ./backend/+docker