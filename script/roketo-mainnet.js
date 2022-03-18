const nearAPI = require('near-api-js')

const FARM_CONTRACT_NAME = "dev-1641987418790-52566958498708"
const main = async () => {
	const keyStore = new nearAPI.keyStores.UnencryptedFileSystemKeyStore(
		`${process.env.HOME}/.near-credentials/`
	)

  // adds the keyPair you created to keyStore
  const NEAR_CONFIG = {
    networkId: "testnet",
    keyStore: keyStore,
    nodeUrl: "https://rpc.testnet.near.org",
    walletUrl: "https://wallet.testnet.near.org"
  }

	const near = await nearAPI.connect({
		deps: {
			keyStore: keyStore,
		},
		...NEAR_CONFIG,
	})

  const account = await near.account(FARM_CONTRACT_NAME)

  const farmContract = new nearAPI.Contract(
    account,
    FARM_CONTRACT_NAME,
    {
      viewMethods: [],
      changeMethods: ["force_upgrade_seed", "migrate"]
    }
  )

  await farmContract.migrate({
    args: {},
    gas: 300000000000000
  })

  const seeds = ["dev-1631277489384-75412609538902$7","dark-only-and-at","dev-1631277489384-75412609538902$3","dev-1631277489384-75412609538902$8","pillars-of-paras-2","asac.near","dev-1631277489384-75412609538902$6","dev-1631277489384-75412609538902$9","dev-1631277489384-75412609538902$1","dev-1631277489384-75412609538902$4","testingdo.testnet","comic-only","dev-1631277489384-75412609538902","dev-1631277489384-75412609538902$5","pillars-of-paras","key-to-paras","hehe.near","dev-1631277489384-75412609538902$2","comic-only-and-at"]

  for (const seed of seeds) {
    await farmContract.force_upgrade_seed({
      args: {
        seed_id: seed
      },
      gas: 300000000000000
    })
  }
}

  

main();

