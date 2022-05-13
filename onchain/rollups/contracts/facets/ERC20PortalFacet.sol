// Copyright 2022 Cartesi Pte. Ltd.

// SPDX-License-Identifier: Apache-2.0
// Licensed under the Apache License, Version 2.0 (the "License"); you may not use
// this file except in compliance with the License. You may obtain a copy of the
// License at http://www.apache.org/licenses/LICENSE-2.0

// Unless required by applicable law or agreed to in writing, software distributed
// under the License is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR
// CONDITIONS OF ANY KIND, either express or implied. See the License for the
// specific language governing permissions and limitations under the License.

/// @title Generic ERC20 Portal facet
pragma solidity ^0.8.0;

import {IERC20} from "@openzeppelin/contracts/token/ERC20/IERC20.sol";

import {IERC20Portal} from "../interfaces/IERC20Portal.sol";

import {LibInput} from "../libraries/LibInput.sol";

contract ERC20PortalFacet is IERC20Portal {
    using LibInput for LibInput.DiamondStorage;

    bytes32 constant INPUT_HEADER = keccak256("ERC20_Transfer");

    /// @notice deposit an amount of a generic ERC20 in the portal and create tokens in L2
    /// @param _ERC20 address of the ERC20 token contract
    /// @param _amount amount of the ERC20 token to be deposited
    /// @param _data information to be interpreted by L2
    /// @return hash of input generated by deposit
    function erc20Deposit(
        address _ERC20,
        uint256 _amount,
        bytes calldata _data
    ) public override returns (bytes32) {
        LibInput.DiamondStorage storage inputDS = LibInput.diamondStorage();
        IERC20 token = IERC20(_ERC20);

        require(
            token.transferFrom(msg.sender, address(this), _amount),
            "ERC20 transferFrom failed"
        );

        bytes memory input = abi.encode(
            INPUT_HEADER,
            msg.sender,
            _ERC20,
            _amount,
            _data
        );

        emit ERC20Deposited(_ERC20, msg.sender, _amount, _data);
        return inputDS.addInternalInput(input);
    }
}
