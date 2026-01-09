// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

/// @notice 一个最简单的可写合约，用于测试发交易/改状态
contract Counter {
    uint256 public number;

    event Increment(address indexed caller, uint256 newNumber);

    function increment() external {
        unchecked {
            number += 1;
        }
        emit Increment(msg.sender, number);
    }
}
