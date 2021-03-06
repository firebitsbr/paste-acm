use iron::headers::ContentType;
use iron::modifiers::Header;
use iron::prelude::*;
use iron::status;
use persistent::Read;
use router::Router;
use rusqlite::Error as RusqliteError;

use snowflake::decode_snowflake;
use DB;

pub fn handler(req: &mut Request) -> IronResult<Response> {
    let mutex = req.get::<Read<DB>>().unwrap();
    let db = mutex.lock().unwrap();

    let id = req.extensions.get::<Router>().unwrap().find("id").unwrap();
    let id = decode_snowflake(&id)
        .map_err(|err| IronError::new(err, status::BadRequest))?;

    let res: Result<String, _> = db.query_row(
        "SELECT data FROM 'paste-acm' WHERE id = ?",
        &[&(id as i64)],
        |row| row.get("data"),
    );
    drop(db);

    match res {
        Ok(body) => {
            let ctype = ContentType::plaintext();
            Ok(Response::with((status::Ok, Header(ctype), body)))
        }
        Err(err) => {
            let status = match err {
                RusqliteError::QueryReturnedNoRows => status::NotFound,
                _ => status::InternalServerError,
            };
            Err(IronError::new(err, status))
        }
    }
}
