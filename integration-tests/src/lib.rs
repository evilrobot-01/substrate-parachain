use frame_support::{sp_io, sp_tracing};
use integration_tests_common::{AccountId, Balance};
use sp_core::{sr25519, storage::Storage, Get};
use sp_runtime::BuildStorage;
use xcm_emulator::{
	decl_test_networks, decl_test_parachains, decl_test_relay_chains, Ancestor,
	BridgeMessageHandler, MultiLocation, ParaId, Parachain, Parent, RelayChain, TestExt, XcmHash,
	X1,
};
use xcm_executor::traits::ConvertLocation;

#[cfg(test)]
mod tests;

// PDD: relay chains
decl_test_relay_chains! {
	// Polkadot
	#[api_version(5)]
	pub struct Polkadot {
		genesis = integration_tests_common::constants::polkadot::genesis(),
		on_init = (),
		// PDD: actual Polkadot runtime
		runtime = {
			Runtime: polkadot_runtime::Runtime,
			RuntimeOrigin: polkadot_runtime::RuntimeOrigin,
			RuntimeCall: polkadot_runtime::RuntimeCall,
			RuntimeEvent: polkadot_runtime::RuntimeEvent,
			MessageQueue: polkadot_runtime::MessageQueue,
			XcmConfig: polkadot_runtime::xcm_config::XcmConfig,
			SovereignAccountOf: polkadot_runtime::xcm_config::SovereignAccountOf,
			System: polkadot_runtime::System,
			Balances: polkadot_runtime::Balances,
		},
		// PDD: pallet type helper
		pallets_extra = {}
	},
	// Kusama
	#[api_version(5)]
	pub struct Kusama {
		genesis = integration_tests_common::constants::kusama::genesis(),
		on_init = (),
		runtime = {
			Runtime: kusama_runtime::Runtime,
			RuntimeOrigin: kusama_runtime::RuntimeOrigin,
			RuntimeCall: kusama_runtime::RuntimeCall,
			RuntimeEvent: kusama_runtime::RuntimeEvent,
			MessageQueue: kusama_runtime::MessageQueue,
			XcmConfig: kusama_runtime::xcm_config::XcmConfig,
			SovereignAccountOf: kusama_runtime::xcm_config::SovereignAccountOf,
			System: kusama_runtime::System,
			Balances: kusama_runtime::Balances,
		},
		pallets_extra = {}
	},
	// Rococo
	#[api_version(5)]
	pub struct Rococo {
		genesis = integration_tests_common::constants::rococo::genesis(),
		on_init = (),
		runtime = {
			Runtime: rococo_runtime::Runtime,
			RuntimeOrigin: rococo_runtime::RuntimeOrigin,
			RuntimeCall: rococo_runtime::RuntimeCall,
			RuntimeEvent: rococo_runtime::RuntimeEvent,
			MessageQueue: rococo_runtime::MessageQueue,
			XcmConfig: rococo_runtime::xcm_config::XcmConfig,
			SovereignAccountOf: rococo_runtime::xcm_config::LocationConverter,
			System: rococo_runtime::System,
			Balances: rococo_runtime::Balances,
		},
		pallets_extra = {}
	}
}

// PDD: parachains
decl_test_parachains! {
	// Parachain A
	pub struct ParaA {
		// PDD: genesis config
		genesis = para_a_genesis(),
		on_init = (),
		// PDD: actual parachain runtime
		runtime = {
			Runtime: para_a_runtime::Runtime,
			RuntimeOrigin: para_a_runtime::RuntimeOrigin,
			RuntimeCall: para_a_runtime::RuntimeCall,
			RuntimeEvent: para_a_runtime::RuntimeEvent,
			XcmpMessageHandler: para_a_runtime::XcmpQueue,
			DmpMessageHandler: para_a_runtime::DmpQueue,
			LocationToAccountId: para_a_runtime::xcm_config::LocationToAccountId,
			System: para_a_runtime::System,
			Balances: para_a_runtime::Balances,
			ParachainSystem: para_a_runtime::ParachainSystem,
			ParachainInfo: para_a_runtime::ParachainInfo,
		},
		pallets_extra = {
			Ping: para_a_runtime::Ping,
		}
	},
	// Parachain B
	pub struct ParaB {
		genesis = para_b_genesis(),
		on_init = (),
		runtime = {
			Runtime: para_b_runtime::Runtime,
			RuntimeOrigin: para_b_runtime::RuntimeOrigin,
			RuntimeCall: para_b_runtime::RuntimeCall,
			RuntimeEvent: para_b_runtime::RuntimeEvent,
			XcmpMessageHandler: para_b_runtime::XcmpQueue,
			DmpMessageHandler: para_b_runtime::DmpQueue,
			LocationToAccountId: para_b_runtime::xcm_config::LocationToAccountId,
			System: para_b_runtime::System,
			Balances: para_b_runtime::Balances,
			ParachainSystem: para_b_runtime::ParachainSystem,
			ParachainInfo: para_b_runtime::ParachainInfo,
		},
		pallets_extra = {}
	}
}

// PDD: define network(s)
decl_test_networks! {
	// Polkadot
	pub struct PolkadotMockNet {
		relay_chain = Polkadot,
		parachains = vec![ ParaA, ParaB, ],
		bridge = ()
	},
	// Kusama
	pub struct KusamaMockNet {
		relay_chain = Kusama,
		parachains = vec![],
		bridge = ()
	},
	// Rococo
	pub struct RococoMockNet {
		relay_chain = Rococo,
		parachains = vec![],
		bridge = ()
	}
}

fn para_a_genesis() -> Storage {
	const PARA_ID: ParaId = ParaId::new(2_000);
	const ED: Balance = para_a_runtime::EXISTENTIAL_DEPOSIT;

	let genesis_config = para_a_runtime::RuntimeGenesisConfig {
		system: para_a_runtime::SystemConfig {
			code: para_a_runtime::WASM_BINARY
				.expect("WASM binary was not build, please build it!")
				.to_vec(),
			..Default::default()
		},
		balances: para_a_runtime::BalancesConfig {
			balances: integration_tests_common::constants::accounts::init_balances()
				.iter()
				.cloned()
				.map(|k| (k, ED * 4096))
				.collect(),
		},
		parachain_info: para_a_runtime::ParachainInfoConfig {
			parachain_id: PARA_ID,
			..Default::default()
		},
		collator_selection: para_a_runtime::CollatorSelectionConfig {
			invulnerables:
				integration_tests_common::constants::collators::invulnerables_asset_hub_polkadot()
					.iter()
					.cloned()
					.map(|(acc, _)| acc)
					.collect(),
			candidacy_bond: ED * 16,
			..Default::default()
		},
		session: para_a_runtime::SessionConfig {
			keys: integration_tests_common::constants::collators::invulnerables()
				.into_iter()
				.map(|(acc, aura)| {
					(
						acc.clone(),                          // account id
						acc,                                  // validator id
						para_a_runtime::SessionKeys { aura }, // session keys
					)
				})
				.collect(),
		},
		polkadot_xcm: para_a_runtime::PolkadotXcmConfig {
			safe_xcm_version: Some(xcm::prelude::XCM_VERSION),
			..Default::default()
		},
		..Default::default()
	};

	genesis_config.build_storage().unwrap()
}

fn para_b_genesis() -> Storage {
	const PARA_ID: ParaId = ParaId::new(2_001);
	const ED: Balance = para_b_runtime::EXISTENTIAL_DEPOSIT;

	let genesis_config = para_b_runtime::RuntimeGenesisConfig {
		system: para_b_runtime::SystemConfig {
			code: para_b_runtime::WASM_BINARY
				.expect("WASM binary was not build, please build it!")
				.to_vec(),
			..Default::default()
		},
		balances: para_b_runtime::BalancesConfig {
			balances: integration_tests_common::constants::accounts::init_balances()
				.iter()
				.cloned()
				.map(|k| (k, ED * 4096))
				.collect(),
		},
		parachain_info: para_b_runtime::ParachainInfoConfig {
			parachain_id: PARA_ID,
			..Default::default()
		},
		collator_selection: para_b_runtime::CollatorSelectionConfig {
			invulnerables:
				integration_tests_common::constants::collators::invulnerables_asset_hub_polkadot()
					.iter()
					.cloned()
					.map(|(acc, _)| acc)
					.collect(),
			candidacy_bond: ED * 16,
			..Default::default()
		},
		session: para_b_runtime::SessionConfig {
			keys: integration_tests_common::constants::collators::invulnerables()
				.into_iter()
				.map(|(acc, aura)| {
					(
						acc.clone(),                          // account id
						acc,                                  // validator id
						para_b_runtime::SessionKeys { aura }, // session keys
					)
				})
				.collect(),
		},
		polkadot_xcm: para_b_runtime::PolkadotXcmConfig {
			safe_xcm_version: Some(xcm::prelude::XCM_VERSION),
			..Default::default()
		},
		..Default::default()
	};

	genesis_config.build_storage().unwrap()
}
