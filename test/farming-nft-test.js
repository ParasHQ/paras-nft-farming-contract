const fs = require('fs');
const assert = require('assert');
const testUtils = require('./test-utils');
const nearAPI = require('near-api-js');
// const {
// 	utils: { format: { parseNearAmount, formatNearAmount } },
// 	transactions: { deployContract, functionCall }
// } = nearAPI;

const {
	gas,
    farmingContractAccount,
    farmingContract,
    farmingContractName,
    nftContract,
    nftContractName,
    ftContract,
    ftContractName,
	getAccount,
	createOrInitAccount,
	getAccountBalance,
} = testUtils;

describe('NFT Farming Contract', function () {
	this.timeout(60000);

    it('should be deployed', async function () {
		const state = await farmingContractAccount.state();
		try {
			await farmingContract.new({
					owner_id: farmingContractName
			});
		} catch (e) {
			if (!/contract has already been initialized/.test(e.toString())) {
				console.warn(e);
			}
		}

		assert.notStrictEqual(state.code_hash, '11111111111111111111111111111111');
	});
})