use std::time::Duration;

fn main() -> nfc1::Result<()> {
    println!("libnfc v{}", nfc1::version());

    let mut context = nfc1::Context::new()?;
    let mut device = context.open_with_connstring("pn532_uart:/dev/ttyUSB0")?;

    println!(
        "NFC device {:?} opened through connection {:?}",
        device.name(),
        device.connstring()
    );
    println!(
        "- Initiator modulations: {:?}",
        device.get_supported_modulation(nfc1::Mode::Initiator)?
    );
    println!(
        "- Target modulations: {:?}",
        device.get_supported_modulation(nfc1::Mode::Target)?
    );

    device.initiator_init()?;
    println!("polling");

    // polling forever
        let result = device.initiator_poll_target(&[
            nfc1::Modulation {
                modulation_type: nfc1::ModulationType::Iso14443a,
                baud_rate: nfc1::BaudRate::Baud106,
            },
            nfc1::Modulation {
                modulation_type: nfc1::ModulationType::Iso14443b,
                baud_rate: nfc1::BaudRate::Baud106,
            },
            nfc1::Modulation {
                modulation_type: nfc1::ModulationType::Felica,
                baud_rate: nfc1::BaudRate::Baud212,
            },

        ], 20, Duration::from_millis(450));

        println!("{:?}", result);

    Ok(())
}
