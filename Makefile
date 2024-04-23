test:
	bats $(shell ls -1 ./tests/*.bats | sort -R)

test_fluvio_install:
	fluvio version
	fluvio topic list
	fluvio topic create foobar
	sleep 3
	echo foo | fluvio produce foobar
	fluvio consume foobar -B -d
