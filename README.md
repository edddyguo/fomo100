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
- 14、产生的奖励可以每天都领取
- 15、解锁是只解锁本金，领钱的接口分为
  - 1、unstake 的赎回接口
  - 2、claim 的 领钱的时候，每天 都 可以 领取，如果不领取就会累积
- 16、每天的领钱的数额要可以调整，管理员接口
- 17、如果用户当天没有领取，要累积到起奖励总额中，也就是经过了多少天，是可以知道的
- 18、每个轮次的总质押数量的确定的，但是 如果 用户 没有 领取，怎么知道历史轮次 所占的比例？？
-

## 思路 2

1、奖励按照 一天作为轮次的粒度，
2、如果是后台去定时 触发的话，可以每个小时在链下，保存每个轮次的的奖池、总质押
、个人质押数量，计算好对应奖励额度，放到后台，
3、用户 claim 的时候，拿着对应的 默克尔证明，去领奖励
4、后台每天都要 更新 树根
5、后台要感知用户 已经 领取了 多少钱，不要重复发奖励
6、要 遍历所有用户的 数据，
这个太麻烦了，需要跑个服务给前端，提供证明数据，还要跑个守护 程序定时更新默克尔树

## 思路 3

1、每天主动开启一个 轮次，有 round_id
2、每个轮次共享，池子 和 用户 的环境变量
3、claim 的时候领取全部轮次的
4、方案否地，新轮次的数据，需要 用户主动触发创建 account 才行

## 思路 4

1、存 最近 7 天的历史记录，系统和个人的
2、后台 放一个管理员程序，定时 触发 处理 每个 阶段的，池子里面的数据
但是 用户的历史数据怎么办？怎么更新用户每个时期可以领取的钱金额？？
用户的历史金额可以再每次 追加的时候，标记，

## 方案 5

1、每 24 小时 进行 一个发奖 轮次，
2、后台定时任务，每天在轮次开始的时候更新改轮次的环境数据（轮次池奖励额度、轮次质押总额）
3、由于 项目要 运行 3 年，这 1000 多天的 历史 快要都要存到合约里面（具体 需要 多少质押待测）
4、用户的 做成支持 100 次追加质押，再多直接报错
5、claim 的时候，进行连续 1000 天的计算统计（这个计算量可能超标）

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


## 创建池子
../target/debug/anchor_cli --prikey 3FiXBX3gPXAMmdYNi6qufpYoCebSmPV3Ua9RMJnvkokJQZher5jDeQJt5y4ksdudFQQd2fDHQ8NNzJXSpsmXMdNd --rpc-url https://api.devnet.solana.com create-pool --program-id 33zLb3sV3rpgaDwzsjHUYBW3SkQCVCaaj1uk7k5juzxQ --token-mint CNyDaZUfYjpn3Epdtp4PAXCaJQ7C2GuSkWgr6NsHoE1h --created-at 1761822450 --round-period-secs 1 --round-reward 10000

//扩展池子空间
//10年的量需要73000(3600*20)的byte，每次10k的申请消耗0.07个sol，这里time给个8次就行了
//记得要提前给这个池子打钱（0.07*8）+ 0.04 = 0.6
../target/debug/anchor_cli --prikey 3FiXBX3gPXAMmdYNi6qufpYoCebSmPV3Ua9RMJnvkokJQZher5jDeQJt5y4ksdudFQQd2fDHQ8NNzJXSpsmXMdNd --rpc-url https://stylish-flashy-scion.solana-devnet.quiknode.pro/440b45854c57eb8ec133590d26123a835cc5a69f expand-pool-state --program-id 33zLb3sV3rpgaDwzsjHUYBW3SkQCVCaaj1uk7k5juzxQ --account Bz2KV5dKiaUQmT4qUb73U3ELtjdzMg3y5A24QhBD6hym --times 8

//单次stake
../target/debug/anchor_cli --prikey 51SH5R65CUANeZLyg4FR4bAVdfYEj8cK2VTGtQqBmyxtTiGFy6nDPQGhd4fGMMzpkFz8SBXvLSKJjz3vCPrSQb16 --rpc-url https://stylish-flashy-scion.solana-devnet.quiknode.pro/440b45854c57eb8ec133590d26123a835cc5a69f stake --program-id 33zLb3sV3rpgaDwzsjHUYBW3SkQCVCaaj1uk7k5juzxQ --token-mint CNyDaZUfYjpn3Epdtp4PAXCaJQ7C2GuSkWgr6NsHoE1h --created-at 1761822449 --round-period-secs 1 --stake-amount 1000000


//多次连续stake,耽搁用户最高100次
for ((i=0;i<=301;i++)); do date;../target/debug/anchor_cli --prikey 51SH5R65CUANeZLyg4FR4bAVdfYEj8cK2VTGtQqBmyxtTiGFy6nDPQGhd4fGMMzpkFz8SBXvLSKJjz3vCPrSQb16 --rpc-url https://api.devnet.solana.com stake --program-id 33zLb3sV3rpgaDwzsjHUYBW3SkQCVCaaj1uk7k5juzxQ --token-mint CNyDaZUfYjpn3Epdtp4PAXCaJQ7C2GuSkWgr6NsHoE1h --created-at 1761822450 --round-period-secs 1 --stake-amount 1000000;sleep 0;done;

//设置每轮的奖金
../target/debug/anchor_cli --prikey 3FiXBX3gPXAMmdYNi6qufpYoCebSmPV3Ua9RMJnvkokJQZher5jDeQJt5y4ksdudFQQd2fDHQ8NNzJXSpsmXMdNd --rpc-url https://api.devnet.solana.com  set-round-reward --program-id 33zLb3sV3rpgaDwzsjHUYBW3SkQCVCaaj1uk7k5juzxQ --token-mint CNyDaZUfYjpn3Epdtp4PAXCaJQ7C2GuSkWgr6NsHoE1h --created-at 1761822450 --round-period-secs 1 --round-reward 20000


//查看pool_state
../target/debug/anchor_cli --prikey 51SH5R65CUANeZLyg4FR4bAVdfYEj8cK2VTGtQqBmyxtTiGFy6nDPQGhd4fGMMzpkFz8SBXvLSKJjz3vCPrSQb16 --rpc-url https://stylish-flashy-scion.solana-devnet.quiknode.pro/440b45854c57eb8ec133590d26123a835cc5a69f pool-state --program-id 33zLb3sV3rpgaDwzsjHUYBW3SkQCVCaaj1uk7k5juzxQ --token-mint CNyDaZUfYjpn3Epdtp4PAXCaJQ7C2GuSkWgr6NsHoE1h --created-at 1761822449 --round-period-secs 1
```

## 流程 交互

```
create_pool -> 用户stake -> 用户claim -> 用户追加stake -> 用户再次claim
-> 用户unlock ->等30天之后->用户unstake
```
