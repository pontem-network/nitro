//
// Copyright 2021, Offchain Labs, Inc. All rights reserved.
//

package arbos

import (
	"math/big"

	"github.com/ethereum/go-ethereum/common"
	"github.com/ethereum/go-ethereum/common/math"
	"github.com/ethereum/go-ethereum/core"
	"github.com/ethereum/go-ethereum/core/vm"
)

var arbAddress = common.HexToAddress("0xabc")

type TxProcessor struct {
	msg          core.Message
	blockContext vm.BlockContext
	stateDB      vm.StateDB
	state        *ArbosState
}

func NewTxProcessor(msg core.Message, evm *vm.EVM) *TxProcessor {
	arbosState := OpenArbosState(evm.StateDB)
	arbosState.SetLastTimestampSeen(evm.Context.Time.Uint64())
	return &TxProcessor{
		msg:          msg,
		blockContext: evm.Context,
		stateDB:      evm.StateDB,
		state:        arbosState,
	}
}

func isAggregated(l1Address, l2Address common.Address) bool {
	return true // TODO
}

func (p *TxProcessor) getAggregator() *common.Address {
	coinbase := p.blockContext.Coinbase
	if isAggregated(coinbase, p.msg.From()) {
		return &coinbase
	}
	return nil
}

func (p *TxProcessor) getExtraGasChargeWei() *big.Int { // returns wei to charge
	return p.state.L1PricingState().GetL1Charges(
		p.msg.From(),
		p.getAggregator(),
		p.msg.Data(),
	)
}

func (p *TxProcessor) getL1GasCharge() uint64 {
	extraGasChargeWei := p.getExtraGasChargeWei()
	gasPrice := p.msg.GasPrice()
	if gasPrice.Cmp(big.NewInt(0)) == 0 {
		// suggest the amount of gas needed for a given amount of ETH is higher in case of congestion
		adjustedPrice := new(big.Int).Mul(p.blockContext.BaseFee, big.NewInt(15))
		adjustedPrice = new(big.Int).Mul(adjustedPrice, big.NewInt(16))
		gasPrice = adjustedPrice
	}
	l1ChargesBig := new(big.Int).Div(extraGasChargeWei, gasPrice)
	if !l1ChargesBig.IsUint64() {
		return math.MaxUint64
	}
	return l1ChargesBig.Uint64()
}

func (p *TxProcessor) InterceptMessage() (*core.ExecutionResult, error) {
	if p.msg.From() != arbAddress {
		return nil, nil
	}
	// Message is deposit
	p.stateDB.AddBalance(*p.msg.To(), p.msg.Value())
	return &core.ExecutionResult{
		UsedGas:    0,
		Err:        nil,
		ReturnData: nil,
	}, nil
}

func (p *TxProcessor) ExtraGasChargingHook(gasRemaining *uint64) error {
	l1Charges := p.getL1GasCharge()
	if *gasRemaining < l1Charges {
		return vm.ErrOutOfGas
	}
	*gasRemaining -= l1Charges
	return nil
}

func (p *TxProcessor) EndTxHook(gasLeft uint64, success bool) error {
	gasUsed := new(big.Int).SetUint64(p.msg.Gas() - gasLeft)
	totalPaid := new(big.Int).Mul(gasUsed, p.msg.GasPrice())
	l1ChargeWei := p.getExtraGasChargeWei()
	l2ChargeWei := new(big.Int).Sub(totalPaid, l1ChargeWei)
	//TODO:
	//	p.stateDB.SubBalance(p.blockContext.Coinbase, l2ChargeWei)
	//	p.stateDB.AddBalance(networkFeeCollector, l2ChargeWei)
	if p.msg.GasPrice().Sign() > 0 {
		// in tests, gasprice coud be 0
		p.state.notifyGasUsed(new(big.Int).Div(l2ChargeWei, p.msg.GasPrice()).Uint64())
	}
	return nil
}
