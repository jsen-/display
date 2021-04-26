set architecture arm

set remotetimeout 100000
target extended-remote localhost:3333


monitor arm semihosting enable
monitor reset run
quit