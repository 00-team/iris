#!/bin/bash

git pull && cargo build -r && rm log.err ; systemctl restart iris
systemctl status iris
