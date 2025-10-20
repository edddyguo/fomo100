```
(cd cli;proxychains4 cargo run -- --bridge-contract-pid 9HiRJw3dYo2MV9B1WrqFfoNjWRPS19mjVDCPqAxuMPfb  --receiver-wallet 677NzkzkDKT9wXDMXGPUvbFp1T7XzJtZZxcRaBAaSvNa  --token-address 7YYNXbfwd5i5scpez18fTkEh2MRHJXoMHPffnWNcpFYf);
anchor build
anchor idl parse -f programs/fomo100/src/lib.rs -o idl/fomo100_go_v1.json
```

# Solana nft mint and issue tool

基于最新的版本创建
https://medium.com/@elchuo160/creating-an-on-chain-nft-on-solana-using-anchor-rust-ipfs-and-quicknode-c92f548b6e0

## Build

```
anchor build
(cd ../;anchor build --arch sbf;anchor deploy)
```

## Deploy

```
anchor deploy
```

## Usage

```
cd cli;proxychains4 cargo run -- --bridge-contract-pid 9HiRJw3dYo2MV9B1WrqFfoNjWRPS19mjVDCPqAxuMPfb  --receiver-wallet 677NzkzkDKT9wXDMXGPUvbFp1T7XzJtZZxcRaBAaSvNa  --token-address 7YYNXbfwd5i5scpez18fTkEh2MRHJXoMHPffnWNcpFYf
```

## Test

```

```

## Todo

- config whitelist for free mint
- mint nft by pay sol or usdt
- get num of allow free mint
- get num of owned
- support config any nft
- support set price

```

1、有初始化 mint，塞一部分免费 mint 的个数
2、支持 sol 和 usdt 两种代币
3、用户自主 mint，不设限制
4、总共两个 nft，也就只有两个图片
5、支持改价格
6、直接做通用的，可以发射各种 nft 项目的通用的合约
7、做好统计接口，查询已 mint 的和待 mint 的数量

```

## test

```
//FUqPtHtbCTK2xmwyWfXiCxdvog5BygEW1ZwNYDMSPyUh
../target/debug/anchor_cli --prikey xxx --rpc-url https://stylish-flashy-scion.solana-devnet.quiknode.pro/440b45854c57eb8ec133590d26123a835cc5a69f mint --minter-program-id  79iwpmjk5mh2acXp2SQxh2JpmNqji76FQQAH4erCCuhu --collection-name "solpen_test_nft_collection_01"

//5wEmePkkXAWYYvvWQDv4Mbenma1jWvzCbt3rK9ihmrqH
../target/debug/anchor_cli --prikey xxx --rpc-url https://stylish-flashy-scion.solana-devnet.quiknode.pro/440b45854c57eb8ec133590d26123a835cc5a69f mint --minter-program-id  79iwpmjk5mh2acXp2SQxh2JpmNqji76FQQAH4erCCuhu --collection-name "solpen_test_nft_collection_01" --pay-sol

//7muWY7LByS4ShDeyVaTCj4MgGuN6DwBacrnDLPwhCAKf
../target/debug/anchor_cli --prikey xxx --rpc-url https://stylish-flashy-scion.solana-devnet.quiknode.pro/440b45854c57eb8ec133590d26123a835cc5a69f create-collection --minter-program-id  79iwpmjk5mh2acXp2SQxh2JpmNqji76FQQAH4erCCuhu --name "Test2 Solpen Core" --symbol "SPC" --uri "https://testapi.solpen.shop/api/main/collection_data" --sol-price 20000000 --settle-token-price 3000000000 --settle-token GeuC5xToMR138PEpP6B2tmXArm42j29Csp8osEQ28Eqc

// elite
../target/debug/anchor_cli --prikey xxx --rpc-url https://stylish-flashy-scion.solana-devnet.quiknode.pro/440b45854c57eb8ec133590d26123a835cc5a69f create-collection --minter-program-id  79iwpmjk5mh2acXp2SQxh2JpmNqji76FQQAH4erCCuhu --name "Test1 Solpen Elite" --symbol "SPE" --uri "https://testapi.solpen.shop/api/main/collection_elite_data" --sol-price 20000000 --settle-token-price 3000000000 --settle-token GeuC5xToMR138PEpP6B2tmXArm42j29Csp8osEQ28Eqc




//set admin
../target/debug/anchor_cli --prikey xxx --rpc-url https://stylish-flashy-scion.solana-devnet.quiknode.pro/440b45854c57eb8ec133590d26123a835cc5a69f set-admin --minter-program-id  79iwpmjk5mh2acXp2SQxh2JpmNqji76FQQAH4erCCuhu --new-admin 7muWY7LByS4ShDeyVaTCj4MgGuN6DwBacrnDLPwhCAKf --new-validator 7muWY7LByS4ShDeyVaTCj4MgGuN6DwBacrnDLPwhCAKf --new-treasurer 7muWY7LByS4ShDeyVaTCj4MgGuN6DwBacrnDLPwhCAKf


//set price
../target/debug/anchor_cli --prikey xxx --rpc-url https://stylish-flashy-scion.solana-devnet.quiknode.pro/440b45854c57eb8ec133590d26123a835cc5a69f set-price --minter-program-id  79iwpmjk5mh2acXp2SQxh2JpmNqji76FQQAH4erCCuhu --collection-name TestName3 --sol-price 12000000 --usdt-price 13000000


../target/debug/anchor_cli --prikey xxx --rpc-url https://stylish-flashy-scion.solana-devnet.quiknode.pro/440b45854c57eb8ec133590d26123a835cc5a69f init-airdrop --minter-program-id  79iwpmjk5mh2acXp2SQxh2JpmNqji76FQQAH4erCCuhu --collection-name "TestCollection5"


// 获取签名

../target/debug/anchor_cli --prikey xxx --rpc-url https://stylish-flashy-scion.solana-devnet.quiknode.pro/440b45854c57eb8ec133590d26123a835cc5a69f sign-airdrop --minter-program-id  79iwpmjk5mh2acXp2SQxh2JpmNqji76FQQAH4erCCuhu --collection-name solpen_test_nft_collection_01 --pubkey FAwGgwJDrwq2SZczfq3rTGemXP9A5iKCoVADPibr9gw9 --amount 1000
```

## todo

- 设计批量签名配置文件
- nft 的元数据用数据库存储

## 参考

- https://github.com/GuidoDipietro/solana-ed25519-secp256k1-sig-verification
- /main/nft_data/3.json

## 资源

### 命名

```
普通 Solpen Core Symbol: SPC
进阶 Solpen Elite  Symbol: SPE
```

### nft

```
https://testapi.solpen.shop/api/main/collection_data
https://testapi.solpen.shop/api/main/nft_data/1.json
```

### elite nft

```
## collection
https://testapi.solpen.shop/api/main/collection_elite_data
https://testapi.solpen.shop/api/main/nft_elite_data/1.json
```

### 后端接口

```
https://app.apifox.com/project/5810805
## 查看总的带申请人信息
https://testapi.solpen.shop/api/accounts/nft_claim_info
## token
eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJkYXRhIjoiRHlrc0dVWjd3cEVoaTZyYUFSUThTRVhDelV1eGJpQld5NDJoRVVyS1RRQmYiLCJpYXQiOjE3NDIwNDEzNTgsImV4cCI6MzkxOTMyMTM1OH0.vDyRAbQjOyP3Cl-xYF30h9KNdFCjZ-707LdiLcciBUo
```

## 更新日志
