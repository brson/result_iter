extern crate result_iter;
use result_iter::{ResultIterExt, MultiError};

use std::io;

#[test]
fn smoke() {
    let err = || Err(io::Error::from(io::ErrorKind::Other));
    let errs = || vec![Ok(()), err(), err(), Ok(())];
    let nerrs = || vec![Ok(()), Ok(()), Ok(()), Ok(())];

    let r: Vec<Result<(), io::Error>>
        = errs().into_iter()
        .end_if_err().collect::<Vec<_>>();

    assert!(r.len() == 2);

    let r: Vec<Result<(), io::Error>>
        = nerrs().into_iter()
        .end_if_err().collect::<Vec<_>>();

    assert!(r.len() == 4);

    let r = || -> Result<(), MultiError<io::Error>> {
        let r: Vec<()>
            = nerrs().into_iter()
            .fail_slow_if_err()?.collect::<Vec<_>>();
        assert!(r.len() == 4);

        let _ = errs().into_iter().fail_slow_if_err()?;
        unreachable!();
    }();

    let e = r.unwrap_err();
    assert!(e.len() == 2);
    let mut e = e.into_iter();
    assert!(e.next().is_some());

    let r = || -> Result<(), io::Error> {
        let r: Vec<()>
            = nerrs().into_iter()
            .fail_fast_if_err()?.collect::<Vec<_>>();
        assert!(r.len() == 4);

        let _ = errs().into_iter().fail_fast_if_err()?;
        unreachable!();
    }();

    assert!(r.is_err());
}
