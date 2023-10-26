#!/bin/bash

# caro send 10 rbtc to alice every 10 second, 1 rbtc as fee
while true
do
    ../wallet/wallet -a http://127.0.0.1:7002 -k ../keys/caro.key transfer_by_id -t 1 -a 10
    sleep 10
done

