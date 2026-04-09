#!/bin/bash

KEYBOARD="-k usb-SINO_WEALTH_USB_KEYBOARD-event-kbd"
THRESHOLD="-t 30"
VERBOSITY="-v 1"
BINARY="/usr/local/bin/double-tap"

if [ -f /etc/default/double-tap ]; then
	. /etc/default/double-tap
fi

exec "$BINARY" $KEYBOARD $THRESHOLD $VERBOSITY
