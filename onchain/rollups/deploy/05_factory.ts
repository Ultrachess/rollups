// Copyright 2022 Cartesi Pte. Ltd.

// Licensed under the Apache License, Version 2.0 (the "License"); you may not
// use this file except in compliance with the License. You may obtain a copy
// of the license at http://www.apache.org/licenses/LICENSE-2.0

// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS, WITHOUT
// WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied. See the
// License for the specific language governing permissions and limitations
// under the License.

import { HardhatRuntimeEnvironment } from "hardhat/types";
import { DeployFunction, DeployOptions } from "hardhat-deploy/types";
import { CartesiDAppFactory } from "../src/types";
import { productionFacetNames, getFacetCuts } from "../src/utils";

const func: DeployFunction = async (hre: HardhatRuntimeEnvironment) => {
    const { deployments, ethers, getNamedAccounts } = hre;
    const { deployer } = await getNamedAccounts();

    const opts: DeployOptions = {
        deterministicDeployment: true,
        from: deployer,
        log: true,
    };

    const { DiamondCutFacet, DiamondInit, Bank } = await deployments.all();

    const facetCuts = await getFacetCuts(productionFacetNames);

    let factoryConfig: CartesiDAppFactory.FactoryConfigStruct = {
        diamondCutFacet: DiamondCutFacet.address,
        diamondInit: DiamondInit.address,
        feeManagerBank: Bank.address,
        diamondCut: facetCuts,
    };

    await deployments.deploy("CartesiDAppFactory", {
        ...opts,
        args: [factoryConfig],
    });
};

export default func;
func.dependencies = ["RollupsFacets", "RollupsInit", "RollupsDiamond", "Bank"];
func.tags = ["RollupsFactory"];
