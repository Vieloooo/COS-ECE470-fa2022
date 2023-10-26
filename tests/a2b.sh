#!/bin/bash

# alce send 10 rbtc to bob every 10 second, 1 rbtc as fee
while true
do
    ../wallet/wallet transfer_by_id -t 1 -a 10
    sleep 10
done


 

