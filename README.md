# solana contract template

## Build

```
anchor build
(anchor build --arch sbf;anchor deploy)
```

## Deploy

```
## dev
anchor deploy

## mainnet
solana program deploy --max-sign-attempts 60 --with-compute-unit-price 75000 target/deploy/fomo100.so --program-id target/deploy/fomo100-keypair.json --keypair ~/.secret/1.json --url https://morning-indulgent-mound.solana-mainnet.quiknode.pro/adb403b775091c1adf4a24c11aacdcb72b50aefe
```

## Todo

- 1、 收益每天用户都可以收取，未解锁前用户不能取出本金
- 2、用户解锁时，会有 30 天的等待时间，等待期间不产生收益
- 3、一个验证接口，可以通过这个接口检查用户质押代币的数量 (补充字段：当前用户的积分值，前端根据公式， pool_vault \* user_point / total_point 来计算预估收益)
- 4、解锁冷静期则禁止再质押（简化需求和逻辑实现,不然还得考虑新的质押是否产生收益这种事情）
- 5、如果没完成，需要增加代理质押功能
- 6、代币精度非标准的 9 位
- 7、项目名称，给个简写？
- 8、质押币种和收益币种一样
- 9、需求理解：收益率按照 时间和金钱来算积分，按照积分比例来瓜分奖池
- 10、时间和币数量的权重是 1:1， 积分的计算公 stake_amount \* stake_duration = stake_point
- 11、用户可领的资金 从 解锁的那个时候 就 开始 锁定
- 12、项目方要考虑，2 天 只有 2 个人 参加，质押了 2 个币的情况，这个情况由于 项目 冷清，低代价 拿走 所有 奖励，为了防止 这种情况，项目 方 可以自己 先抵押一些钱，或者都不用 考虑
- 13、项目 没有 结束时间，只有有人还在质押 中，他就一直有份额，就 不会 取空池子，就能一直玩下去
- 14、

## 接口设计

- 1、stake 充币质押
  - 质押的本金放在 seed 为（"user_vault"+user_state_pda）的 pda 地址中
  - 奖池的钱放在 seed 为 （"user_vault"+pool_state_pda）的 pda 地址中
- 2、unlock 解除质押
  - 开始在计算上划拨资金池里面的钱到已解锁未 申领的 类别中
- 3、claim 领取质押币和收益
- 在 unlock 的 时间达到后进行 解锁质押
- 4、pool_state 资金池状态（总积分、奖池总额、 质押总额）
- 5、user_state 用户质押状态（用户积分、用户累计质押金额）

## notes

1、每次新的项目需要用新的 私钥来 部署

## common cmd

```
# 生产新地址(部署生产之前的私钥创建)
solana-keygen new -o 1.json
# 查合约地址
solana address -k 1.json

```

## cli tool

```

../target/debug/anchor_cli --prikey xxx --rpc-url https://stylish-flashy-scion.solana-devnet.quiknode.pro/440b45854c57eb8ec133590d26123a835cc5a69f mint --minter-program-id 79iwpmjk5mh2acXp2SQxh2JpmNqji76FQQAH4erCCuhu --collection-name "solpen_test_nft_collection_01"

../target/debug/anchor_cli --prikey xxx --rpc-url https://stylish-flashy-scion.solana-devnet.quiknode.pro/440b45854c57eb8ec133590d26123a835cc5a69f mint --minter-program-id 79iwpmjk5mh2acXp2SQxh2JpmNqji76FQQAH4erCCuhu --collection-name "solpen_test_nft_collection_01" --pay-sol

```
