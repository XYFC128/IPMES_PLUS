#!/bin/bash
# Forward

diff -Naur <(git show 09b743c89cd8d5cb0:./join_layer/sub_pattern_buffer.rs) <(git show HEAD:./join_layer/sub_pattern_buffer.rs) > forward.patch
sed -i 's|/dev/fd/[0-9]\+|./join_layer/sub_pattern_buffer.rs|g' forward.patch

diff -Naur ./unoptimized_join_layer.rs ./join_layer.rs >> forward.patch

# Backward
diff -Naur <(git show HEAD:./join_layer/sub_pattern_buffer.rs) <(git show 09b743c89cd8d5cb0:./join_layer/sub_pattern_buffer.rs) > backward.patch
sed -i 's|/dev/fd/[0-9]\+|./join_layer/sub_pattern_buffer.rs|g' backward.patch

diff -Naur ./join_layer.rs ./unoptimized_join_layer.rs >> backward.patch
