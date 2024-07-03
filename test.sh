#!/bin/bash
assert() {
	expected="$1"
	input="$2"

	./chibicc "$input" >tmp.s || exit
    gcc -static -o tmp tmp.s
	./tmp
	actual="$?"

	if [ "$actual" = "$expected" ]; then
		echo "$input => $actual"
	else
		echo "$input => $expected expected, but got $actual"
		exit 1
	fi
}

cargo build
mv target/debug/chibicc_rust chibicc

assert 0 0
assert 42 42

echo OK
