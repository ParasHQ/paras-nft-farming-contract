const fs = require('fs')
const assert = require('assert')
const testUtils = require('./test-utils')
const nearAPI = require('near-api-js')
const JSBI = require('jsbi')
const { error } = require('console')

const {
	gas,
	gas_max,
	farmingContractAccount,
	farmingContract,
	farmingContractName,
	nftContract,
	nftContractName,
	ftContract,
	ftContractName,
	ownerAccount,
	ownerAccountName,
	getAccount,
	createOrInitAccount,
	getAccountBalance,
} = testUtils

function sleep(ms) {
	return new Promise((resolve) => setTimeout(resolve, ms))
}

describe.only('Locked Seed', function () {
	this.timeout(60000)

	it('should be deployed', async function () {
		const state = await farmingContractAccount.state()
		try {
			await farmingContract.new({
				owner_id: farmingContractName,
			})
		} catch (e) {
			if (!/contract has already been initialized/.test(e.toString())) {
				console.warn(e)
			}
		}

		assert.notStrictEqual(state.code_hash, '11111111111111111111111111111111')
	})

	it('should create simple farm ft staking', async function () {
		try {
			const seed = await farmingContract.get_seed_info({
				seed_id: ftContractName,
			})
			const farm = await farmingContract.get_farm({
				farm_id: `${ftContractName}#0`,
			})
			console.log(seed)
			console.log(farm)
			if (seed) return
			await farmingContract.create_simple_farm(
				{
					terms: {
						seed_id: ftContractName,
						reward_token: ftContractName,
						start_at: 0,
						reward_per_session: '1000000000000000000',
						session_interval: 60,
					},
					metadata: {
						title: 'PARAS Staking',
						media:
							'https://paras-cdn.imgix.net/bafybeidoerucqfzyazvyfm5axjixs6vie7ts2myru7g5mu2ub7tlvixpqq?w=800',
					},
				},
				gas,
				(deposit = '19380000000000000000000')
			)
		} catch (e) {
			console.warn(e)
		}
	})

	it('should ft storage deposit', async function () {
		try {
			await farmingContractAccount.functionCall({
				contractId: ftContractName,
				methodName: 'storage_deposit',
				args: {
					accountId: farmingContractName,
				},
				gas: gas,
				attachedDeposit: '1250000000000000000000',
			})
		} catch (e) {
			console.warn(e)
		}
	})

	it('should farming storage deposit', async function () {
		try {
			await ownerAccount.functionCall({
				contractId: farmingContractName,
				methodName: 'storage_deposit',
				args: {
					accountId: ownerAccountName,
				},
				gas: gas,
				attachedDeposit: '100000000000000000000000',
			})
		} catch (e) {
			console.warn(e)
		}
	})

	it('should ft stake', async function () {
		try {
			const user_seeds = await farmingContract.list_user_seeds({
				account_id: ownerAccountName,
			})

			if (
				user_seeds[ftContractName] &&
				parseInt(user_seeds[ftContractName]) > 10000000000000000000n
			)
				return

			await ftContract.ft_transfer_call({
				args: {
					receiver_id: farmingContractName,
					amount: '100000000000000000000',
					msg: '',
				},
				gas: gas,
				amount: '1',
			})
		} catch (e) {
			console.warn(e)
		}
	})

	// pause, upgrade to new contract

	it('should ft lock balance', async function () {
		try {
			await ownerAccount.functionCall({
				contractId: farmingContractName,
				methodName: 'lock_ft_balance',
				args: {
					seed_id: ftContractName,
					amount: '50000000000000000000', // 50
					duration: 1, // lock 1 second
				},
				gas: gas,
				attachedDeposit: '1',
			})
			const user_locked_seeds = await farmingContract.list_user_locked_seeds({
				account_id: ownerAccountName,
			})
			if (user_locked_seeds[ftContractName].balance !== '50000000000000000000') {
				throw Error('total locked seed is not the same')
			}
			await sleep(2) // sleep 2 second
		} catch (e) {
			console.warn(e)
		}
	})

	it('should ft unlock balance', async function () {
		try {
			await ownerAccount.functionCall({
				contractId: farmingContractName,
				methodName: 'unlock_ft_balance',
				args: {
					seed_id: ftContractName,
				},
				gas: gas,
				attachedDeposit: '1',
			})
			const user_locked_seeds = await farmingContract.list_user_locked_seeds({
				account_id: ownerAccountName,
			})
			if (user_locked_seeds[ftContractName]) {
				throw Error('locked seed still exsist after the user unlock balance')
			}
		} catch (e) {
			console.warn(e)
		}
	})

	it('should ft unstake', async function () {
		try {
			const ownerAccountBalance = await ftContract.ft_balance_of({
				account_id: ownerAccountName,
			})
			console.log('ownerAccountBalance', ownerAccountBalance)

			await ownerAccount.functionCall({
				contractId: farmingContractName,
				methodName: 'withdraw_seed',
				args: {
					seed_id: ftContractName,
					amount: '10000000000000000000',
				},
				gas: gas,
				attachedDeposit: '1',
			})

			const ownerAccountBalanceAfter = await ftContract.ft_balance_of({
				account_id: ownerAccountName,
			})

			if (
				JSBI.lessThan(
					JSBI.subtract(JSBI.BigInt(ownerAccountBalanceAfter), JSBI.BigInt(ownerAccountBalance)),
					JSBI.BigInt('10000000000000000000')
				)
			)
				throw Error('amount not the same')
		} catch (e) {
			console.warn(e)
		}
	})
})
