#!/bin/zsh

cargo build
./target/debug/client set hello world
./target/debug/client get hello
./target/debug/client set hello mom
./target/debug/client get hello
