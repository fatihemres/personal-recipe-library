use crate::{
    domain::{AppError, Bootstrap, Preferences},
    persistence::Database,
};
use std::{path::PathBuf, sync::mpsc};
use tokio::sync::oneshot;

type StorageJob = Box<dyn FnOnce(Result<&mut Database, AppError>) + Send>;

enum Request {
    Bootstrap(oneshot::Sender<Result<Bootstrap, AppError>>),
    Execute(StorageJob),
    Save(Preferences, oneshot::Sender<Result<Preferences, AppError>>),
}
pub struct StorageService {
    sender: mpsc::Sender<Request>,
}
impl StorageService {
    pub fn new(directory: Result<PathBuf, AppError>) -> Self {
        Self::start(directory, true)
    }
    #[cfg(test)]
    pub(crate) fn without_catalog(directory: Result<PathBuf, AppError>) -> Self {
        Self::start(directory, false)
    }
    fn start(directory: Result<PathBuf, AppError>, install_catalog: bool) -> Self {
        let (sender, receiver) = mpsc::channel();
        std::thread::spawn(move || {
            let mut db: Option<Database> = None;
            for request in receiver {
                let database = match &mut db {
                    Some(database) => Ok(database),
                    None => match directory.as_ref().map_err(Clone::clone).and_then(|p| {
                        let mut database = Database::open(p)?;
                        if install_catalog {
                            database.install_bundled_catalog()?;
                        }
                        Ok(database)
                    }) {
                        Ok(database) => {
                            db = Some(database);
                            Ok(db.as_mut().expect("database assigned"))
                        }
                        Err(error) => Err(error),
                    },
                };
                match request {
                    Request::Execute(job) => job(database),
                    Request::Bootstrap(reply) => {
                        let _ = reply.send(database.and_then(|d| d.bootstrap()));
                    }
                    Request::Save(preferences, reply) => {
                        let _ = reply.send(database.and_then(|d| d.save_preferences(preferences)));
                    }
                }
            }
        });
        Self { sender }
    }
    pub async fn execute<T: Send + 'static>(
        &self,
        operation: impl FnOnce(&mut Database) -> Result<T, AppError> + Send + 'static,
    ) -> Result<T, AppError> {
        let (tx, rx) = oneshot::channel();
        self.sender
            .send(Request::Execute(Box::new(move |database| {
                let _ = tx.send(database.and_then(operation));
            })))
            .map_err(|_| AppError::storage())?;
        rx.await.map_err(|_| AppError::storage())?
    }
    pub async fn bootstrap(&self) -> Result<Bootstrap, AppError> {
        let (tx, rx) = oneshot::channel();
        self.sender
            .send(Request::Bootstrap(tx))
            .map_err(|_| AppError::storage())?;
        rx.await.map_err(|_| AppError::storage())?
    }
    pub async fn save(&self, preferences: Preferences) -> Result<Preferences, AppError> {
        preferences.validate()?;
        let (tx, rx) = oneshot::channel();
        self.sender
            .send(Request::Save(preferences, tx))
            .map_err(|_| AppError::storage())?;
        rx.await.map_err(|_| AppError::storage())?
    }
}
