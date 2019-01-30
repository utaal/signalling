
use npnc::bounded::spsc;

fn main() {

    let (producers, consumers): (Vec<_>, Vec<_>) = (0..16).map(|_| spsc::channel(2048)).unzip();
    let (signaller, signalled) = signalling::signal();

    let receiver = std::thread::spawn(move || {
        let mut consumers: Vec<_> = consumers.into_iter().map(Some).collect();
        let signalled = signalled.this_thread();

        while !consumers.is_empty() {
            let mut had_data = false;
            for c in consumers.iter_mut() {
                loop {
                    match c.as_mut().expect("Some").consume() {
                        Ok(_) => {
                            had_data = true;
                            continue;
                        },
                        Err(npnc::ConsumeError::Disconnected) => {
                            *c = None;
                            break;
                        },
                        Err(npnc::ConsumeError::Empty) => {
                            break;
                        }
                    }
                }
            }
            consumers.retain(|x| x.is_some());
            if !had_data {
                signalled.wait();
            }
        }
    });

    let senders: Vec<_> = producers.into_iter().enumerate().map(|(i, p)| {
        let signaller = signaller.clone();
        std::thread::spawn(move || {
            std::thread::sleep(std::time::Duration::from_millis(i as u64 * 64));
            for round in 0..256 {
                dbg!((i, round));
                if round % 16 == 0 {
                    std::thread::sleep(std::time::Duration::from_millis(2048));
                }
                for x in 0..10 {
                    p.produce(x).expect("queue full");
                }
                signaller.ping();
            }
        })
    }).collect();

    receiver.join().unwrap();
    for s in senders.into_iter() {
        s.join().unwrap();
    }

}
