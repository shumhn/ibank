#!/bin/bash
# Clean build script that suppresses WebSocket connection warnings

arcium build 2>&1 | grep -v "ws error"
