use super::Database;
use crate::domain::{AppError, Preferences, Theme};
impl Database {
    pub fn preferences(&self) -> Result<Preferences, AppError> {
        let (theme, locale): (String, String) = self.conn.query_row(
            "SELECT theme, locale FROM preferences WHERE id = 1",
            [],
            |r| Ok((r.get(0)?, r.get(1)?)),
        )?;
        let prefs = Preferences {
            theme: Theme::parse(&theme)?,
            locale,
        };
        prefs.validate().map_err(|_| AppError::integrity())?;
        Ok(prefs)
    }
    pub fn save_preferences(&mut self, preferences: Preferences) -> Result<Preferences, AppError> {
        preferences.validate()?;
        let tx = self.conn.transaction()?;
        let count = tx.execute("UPDATE preferences SET theme=?1, locale=?2, updated_at=strftime('%Y-%m-%dT%H:%M:%fZ', 'now') WHERE id=1",
            rusqlite::params![preferences.theme.as_str(), preferences.locale])?;
        if count != 1 {
            return Err(AppError::integrity());
        }
        tx.commit()?;
        Ok(preferences)
    }
}
