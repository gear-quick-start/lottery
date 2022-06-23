#[cfg(test)]
extern crate std;
#[cfg(test)]
use std::println;
use std::time::{SystemTime, UNIX_EPOCH};

use codec::Encode;
use gstd::BTreeMap;
use gtest::{Program, System};
use lt_io::*;
const USERS: &[u64] = &[3, 4, 5];

fn init(sys: &System) {
    sys.init_logger();

    let lt = Program::current(sys);

    let res = lt.send_bytes_with_value(USERS[0], b"Init", 10000);

    assert!(res.log().is_empty());
}

#[test]
fn start_lottery() {
    let time = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards")
        .as_secs();

    let state = LtEvent::LotteryState {
        lottery_owner: USERS[0].into(),
        lottery_started: true,
        lottery_start_time: time,
        lottery_duration: 5000,
        participation_cost: 1000,
        prize_fund: 2000,
        token_address: None,
        lottery_id: 1,
        players: BTreeMap::new(),
        lottery_history: BTreeMap::new(),
    };

    let sys = System::new();
    init(&sys);
    let lt = sys.get_program(1);

    let res = lt.send(
        USERS[0],
        LtAction::StartLottery {
            duration: 5000,
            token_address: None,
            participation_cost: 1000,
            prize_fund: 2000,
        },
    );
    assert!(res.log().is_empty());

    println!("time: {}", time);

    let res = lt.send(USERS[0], LtAction::LotteryState);
    assert!(res.contains(&(USERS[0], state.encode())));
}

#[test]
fn enter() {
    let sys = System::new();
    init(&sys);
    let lt = sys.get_program(1);

    let res = lt.send(
        USERS[0],
        LtAction::StartLottery {
            duration: 5000,
            token_address: None,
            participation_cost: 1000,
            prize_fund: 2000,
        },
    );
    assert!(res.log().is_empty());

    let res = lt.send_with_value(USERS[0], LtAction::Enter(1000), 1000);
    assert!(res.contains(&(USERS[0], LtEvent::PlayerAdded(0).encode())));

    let res = lt.send_with_value(USERS[1], LtAction::Enter(1000), 1000);
    assert!(res.contains(&(USERS[1], LtEvent::PlayerAdded(1).encode())));
}

#[test]
fn pick_winner() {
    let sys = System::new();
    init(&sys);
    let lt = sys.get_program(1);

    let res = lt.send(
        USERS[0],
        LtAction::StartLottery {
            duration: 5000,
            token_address: None,
            participation_cost: 1000,
            prize_fund: 2000,
        },
    );
    assert!(res.log().is_empty());

    let res = lt.send_with_value(USERS[0], LtAction::Enter(1000), 1000);
    assert!(res.contains(&(USERS[0], LtEvent::PlayerAdded(0).encode())));

    let res = lt.send_with_value(USERS[1], LtAction::Enter(1000), 1000);
    assert!(res.contains(&(USERS[1], LtEvent::PlayerAdded(1).encode())));

    sys.spend_blocks(5000);

    let res = lt.send(USERS[0], LtAction::PickWinner);

    println!("Winner index: {:?}", res.decoded_log::<LtEvent>());
    assert!(
        res.contains(&(USERS[0], LtEvent::Winner(0).encode()))
            || res.contains(&(USERS[0], LtEvent::Winner(1).encode()))
    );
}

#[test]
fn reset_lottery() {
    let sys = System::new();
    init(&sys);
    let lt = sys.get_program(1);

    let res = lt.send(
        USERS[0],
        LtAction::StartLottery {
            duration: 5000,
            token_address: None,
            participation_cost: 1000,
            prize_fund: 2000,
        },
    );
    assert!(res.log().is_empty());

    let res = lt.send_with_value(USERS[0], LtAction::Enter(1000), 1000);
    assert!(res.contains(&(USERS[0], LtEvent::PlayerAdded(0).encode())));

    let res = lt.send_with_value(USERS[1], LtAction::Enter(1000), 1000);
    assert!(res.contains(&(USERS[1], LtEvent::PlayerAdded(1).encode())));

    let res = lt.send(USERS[0], LtAction::ResetLottery);
    assert!(res.contains(&(USERS[0], LtEvent::Reset.encode())));

    let state = LtEvent::LotteryState {
        lottery_owner: USERS[0].into(),
        lottery_id: 1,
        lottery_started: false,
        lottery_start_time: 0,
        lottery_duration: 0,
        participation_cost: 0,
        prize_fund: 0,
        token_address: None,
        players: BTreeMap::new(),
        lottery_history: BTreeMap::new(),
    };

    let res = lt.send(USERS[0], LtAction::LotteryState);
    assert!(res.contains(&(USERS[0], state.encode())));
}
