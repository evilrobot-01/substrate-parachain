use super::*;
use cumulus_ping::Event;
use frame_support::assert_ok;
use xcm_emulator::assert_expected_events;

type Pings = cumulus_ping::Pings<<ParaA as Parachain>::Runtime>;

#[allow(dead_code)]
fn overview() {
	// PDD: network declaration
	type MockNetwork = PolkadotMockNet;
	type RelayChain = Polkadot;
	type Para = ParaA;

	// PDD: under the hood of xcm-emulator
	type TestExt = dyn xcm_emulator::TestExt;

	// PDD: xcm-emulator macros: "macro_rules! decl_test_networks"

	// PDD: xcm-emulator message queues etc.
	let _messages = xcm_emulator::DOWNWARD_MESSAGES;

	// PDD: use-case components: ping
	type ParaARuntime = <ParaA as Parachain>::Runtime;
	type Ping = cumulus_ping::Pallet<ParaARuntime>;
	type ParaBRuntime = <ParaB as Parachain>::Runtime;
}

// Ensure para A can ping para B
#[test]
fn ping_pong() {
	// PDD: initialise tracing
	init_tracing();

	// Send ping from para A to B
	// PDD: execute_with impl > "macro_rules! __impl_test_ext_for_parachain"
	let pings = ParaA::execute_with(|| {
		type RuntimeEvent = <ParaA as Parachain>::RuntimeEvent;
		type RuntimeOrigin = <ParaA as Parachain>::RuntimeOrigin;

		// PDD: send
		assert_ok!(<ParaA as ParaAPallet>::Ping::send(
			RuntimeOrigin::root(),
			ParaB::para_id(),
			vec![]
		));
		assert_expected_events!(
			ParaA,
			vec![ RuntimeEvent::Ping(Event::PingSent (para_id, ..)) => { para_id: para_id == &ParaB::para_id(), }, ]
		);

		// PDD: .execute_with(..) can return results
		Pings::iter_keys().count()
	});

	// Ensure para B pinged, pong sent back to A
	ParaB::execute_with(|| {
		type RuntimeEvent = <ParaB as Parachain>::RuntimeEvent;

		assert_expected_events!(
			ParaB,
			vec![
				RuntimeEvent::Ping(Event::Pinged (para_id, ..)) => { para_id: para_id == &ParaA::para_id(), },
				RuntimeEvent::Ping(Event::PongSent (para_id, ..)) => { para_id: para_id == &ParaA::para_id(), },
			]
		);
	});

	// Ensure para A ponged
	ParaA::execute_with(|| {
		type RuntimeEvent = <ParaA as Parachain>::RuntimeEvent;

		assert_expected_events!(
			ParaA,
			vec![ RuntimeEvent::Ping(Event::Ponged (para_id, ..)) => { para_id: para_id == &ParaB::para_id(), }, ]
		);

		// Ensure successful ping removed
		assert_eq!(Pings::iter_keys().count(), pings - 1)
	});
}

static INIT: std::sync::Once = std::sync::Once::new();
fn init_tracing() {
	INIT.call_once(|| {
		// Add test tracing (from sp_tracing::init_for_tests()) but filtering for xcm logs only
		let _ = tracing_subscriber::fmt()
			.with_max_level(tracing::Level::TRACE)
			// PDD: filter tracing output
			// Comment out this line to see all traces
			.with_env_filter(
				vec![
					"xcm=trace",
					// PDD: xcm-emulator
					"events=trace",
					"hrmp=trace",
					"dmp=trace",
					"ump=trace",
					// PDD: ping pallet
					"ping=trace",
				]
				.join(","),
			)
			.with_test_writer()
			.init();
	});
}
