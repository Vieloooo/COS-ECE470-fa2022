#!/bin/bash

# bob    send 10 rbtc to caro every 10 second, 1 rbtc as fee
while true
do
    ../wallet/wallet -a http://127.0.0.1:7001 -k ../keys/bob.key transfer_by_id -t 2 -a 10
    sleep 10
done


 

