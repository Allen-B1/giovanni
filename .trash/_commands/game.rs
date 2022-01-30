use std::fmt;

use super::super::palette;

#[derive(Clone, Copy)]
pub enum Speed {
    P25,
    P5,
    P75,
    S1,
    S1P5,
    S2,
    S3,
    S4
}

#[derive(Debug)]
pub struct SpeedError(pub String);

impl std::error::Error for SpeedError {}
impl fmt::Display for SpeedError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl fmt::Display for Speed {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}x", self.as_num())
    }
}

impl Speed {
    fn as_num(self) -> &'static str {
        match self {
            Speed::P25 => "0.25",
            Speed::P5 => "0.5",
            Speed::P75 => "0.75",
            Speed::S1 => "1",
            Speed::S1P5 => "1.5x+",
            Speed::S2 => "2",
            Speed::S3 => "3",
            Speed::S4 => "4"            
        }
    }
}

impl std::str::FromStr for Speed {
    type Err = SpeedError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s {
            "0.25" | "0.25x "=> Speed::P25,
            "0.5" | "0.5x" => Speed::P5,
            "0.75" | "0.75x" => Speed::P75,
            "1" | "1.0" | "1x" | "1.0x" => Speed::S1,
            "1.5" | "1.5x" => Speed::S1P5,
            "2" | "2.0" | "2x" | "2.0x" => Speed::S2,
            "3" | "3.0" | "3x" | "3.0x" => Speed::S3,
            "4" | "4.0" | "4x" | "4.0x" => Speed::S4,
            _ => return Err(SpeedError(format!("invalid speed '{}'", s).into()))
        })
    }
}

/*
#[poise::command(slash_command, prefix_command)]
pub async fn custom(ctx: Context<'_>,
    #[description = "Custom room ID"] code: Option<String>,
    #[description = "Map"] map: Option<String>,
    #[description = "Game speed"] speed: Option<Speed>,
) -> Result<(), Error> {
    let code = code.unwrap_or_else(|| {
        std::iter::repeat(())
            .map(|_| rand::thread_rng().sample(rand::distributions::Alphanumeric))
            .map(char::from)
            .take(4)
            .collect::<String>()
    });
    let mut url = match code.as_str() {
        "main" => "https://generals.io/?queue=main".to_string(),
        "1v1" =>  "https://generals.io/?queue=1v1".to_string(),
        "2v2" =>  "https://generals.io/teams/matchmaking".to_string(),
        _ =>  format!("https://generals.io/games/{}", code)
    };
    let mut options: Vec<String> = Vec::new();
    if let Some(map) = &map {
        options.push(format!("map={}", urlencoding::encode(map)))
    }
    if let Some(speed) = speed {
        options.push(format!("speed={}", speed.as_num()))
    }
    if options.len() != 0 {
        url += "?";
        url +=  &options.join("&");
    }

    let mut description = url.clone();
    if options.len() != 0 {
        description += "\n";
    }
    if let Some(map) = &map {
        description += "\n";
        description += &format!("**Map:** {}", map);
    }
    if let Some(speed) = speed {
        description += "\n";
        description += &format!("**Speed:** {}", speed);
    }

    poise::send_reply(ctx, |f| f
        .embed(|f| f
            .title("Custom Game")
            .url(url)
            .description(description)
            .colour(palette::EMBED_GAME)
            .footer(|f| 
                f.text("TODO database"))
        )
    ).await?;

    Ok(())
}
*/