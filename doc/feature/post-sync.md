# Post sync

Sync posts between TW and JP servers.

## Algorithm: A naive implementation

We compare difference between their posts every day to find minimal and maximal difference.
Then use a 7-day moving average to smooth them.

