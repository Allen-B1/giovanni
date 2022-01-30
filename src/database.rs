use sqlx::{Row, Executor};


pub struct Database {
    pool: sqlx::sqlite::SqlitePool
}

impl Database {
    pub async fn new(pool: sqlx::sqlite::SqlitePool) -> Result<Self, Box<dyn std::error::Error>> {
        sqlx::query("CREATE TABLE IF NOT EXISTS usernames (discord INT, username TEXT) ").execute(&pool).await?;

        Ok(Self {
            pool
        })
    }

    pub async fn get_username(&self, discord: u64) -> Result<Option<String>, Box<dyn std::error::Error>> {
        let data = sqlx::query("SELECT username FROM usernames WHERE discord = ?")
            .bind(discord as i64)
            .fetch_optional(&self.pool)
            .await?
            .map(|x| x.get("username"));

        
        Ok(data)
    }

    pub async fn get_discord(&self, username: &str) -> Result<Option<u64>, Box<dyn std::error::Error>> {
        let data: Option<i64> = sqlx::query("SELECT discord FROM usernames WHERE username = ?")
            .bind(username)
            .fetch_optional(&self.pool)
            .await?
            .map(|x| x.get("discord"));
        
        Ok(data.map(|x| x as u64))
    }

    /// Assumes the entry does not exist for either discord or username.
    pub async fn add_username(&self, discord: u64, username: &str) -> Result<(), Box<dyn std::error::Error>> { 
        sqlx::query("INSERT INTO usernames VALUES (?, ?)")
            .bind(discord as i64)
            .bind(username)
            .execute(&self.pool)
            .await?;

        Ok(())
    }
}