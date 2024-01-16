use rand::seq::SliceRandom;
use rand::thread_rng;
use serde::{Deserialize, Serialize};
use serde_json;
use std::io::prelude::*;
use std::net::{TcpListener, TcpStream};
//Definir as mensagens do servidor aqui
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Mao {
    pub mao: Vec<Carta>,
}

#[derive(Debug, Serialize, Deserialize)]
enum ServerMessage {
    START(String),
    SendPlayerCards(Player),
    Wait,
    PlayedCard(Carta),
    Update(GameStats),
    GameOver(u16, GameStats),
    TrucoRequest,
    TrucoResponse(u8),
}

#[derive(Debug, Serialize, Deserialize)]
enum ClientMessage {
    Connect(String),
    GetCards,
    PlayCard((Carta, u16)),
    GetResponse,
    Truco(ServerMessage),
    TrucoResponse((u8, u16)),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameStats {
    last_winner_id: u16,
    last_played_card: Carta,
    played_cards: Vec<Carta>,
    draw: bool,
    end_turn: bool,
    end_round: bool,
    p1_id: u16,
    p2_id: u16,
    p1_points: (u16, u16), // (round_points, total_points)
    p2_points: (u16, u16), // (round_points, total_points)
    gameover: bool,
    turn: u16,
}
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Player {
    is_turn: bool,
    id: u16,
    cartas: Mao,
    manilha: Carta,
    pode_trucar: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialOrd, PartialEq)]
pub enum Naipe {
    None,
    Ouros,
    Espadas,
    Copas,
    Paus,
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialOrd, PartialEq)]
pub enum Rank {
    None,
    Quatro,
    Cinco,
    Seis,
    Sete,
    J,
    Q,
    K,
    As,
    Dois,
    Tres,
}
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Carta {
    pub naipe: Naipe, //remover esse pub e adicionar getters e setters
    pub rank: Rank,   //remover esse pub e adicionar getters
}

impl Rank {
    fn next(&self) -> Rank {
        match self {
            Rank::None => Rank::None,
            Rank::Quatro => Rank::Cinco,
            Rank::Cinco => Rank::Seis,
            Rank::Seis => Rank::Sete,
            Rank::Sete => Rank::J,
            Rank::J => Rank::Q,
            Rank::Q => Rank::K,
            Rank::K => Rank::As,
            Rank::As => Rank::Dois,
            Rank::Dois => Rank::Tres,
            Rank::Tres => Rank::Quatro,
        }
    }
}

fn cria_baralho() -> Vec<Carta> {
    let mut cards = Vec::new();
    for naipe in [Naipe::Copas, Naipe::Ouros, Naipe::Paus, Naipe::Espadas] {
        for rank in [
            Rank::As,
            Rank::Dois,
            Rank::Tres,
            Rank::Quatro,
            Rank::Cinco,
            Rank::Seis,
            Rank::Sete,
            Rank::J,
            Rank::Q,
            Rank::K,
        ] {
            cards.push(Carta {
                naipe: naipe.clone(),
                rank: rank.clone(),
            });
        }
    }
    // Embaralhar o Baralho
    let mut rng = thread_rng();
    cards.shuffle(&mut rng);
    cards
}

fn process_client_message(stream: &TcpStream) -> ClientMessage {
    let mut buffer = [0; 1024];
    let bytes_read = stream.try_clone().unwrap().read(&mut buffer).unwrap();
    let received_msg = serde_json::from_slice(&buffer[..bytes_read]).unwrap();
    received_msg
}
fn main() {
    let server_ip: &str = "127.0.0.1:7878";
    let serialize = |msg| serde_json::to_string(&msg).unwrap();

    let mut played_cards = Vec::new();

    let mut round_value = 1;

    let mut players = Vec::new();
    let mut streams = Vec::new();
    let mut player_names = Vec::new();
    let mut truco_state_list = Vec::new();

    // let qnt_players = 2;

    //Substituir pelo ip local
    let listen = TcpListener::bind(server_ip).unwrap(); //

    let mut baralho;
    let mut manilha = Carta {
        rank: Rank::None,
        naipe: Naipe::None,
    };

    let mut game_stats = GameStats {
        p1_id: 1,
        p2_id: 2,
        last_winner_id: 0,
        draw: false,
        end_turn: false,
        end_round: false,
        p1_points: (0, 0),
        p2_points: (0, 0),
        gameover: false,
        turn: 0,
        played_cards: vec![Carta {
            naipe: Naipe::None,
            rank: Rank::None,
        }],
        last_played_card: manilha.clone(),
    };

    for stream in listen.incoming() {
        match stream {
            Ok(stream) => {
                // let processed_message = process_client_message(&stream);

                match process_client_message(&stream) {
                    ClientMessage::Connect(name) => {
                        if streams.len() == 0 {
                            streams.push(stream.try_clone().unwrap());
                            player_names.push(name);
                        } else {
                            player_names.push(name.clone());
                            let mut p1_stream = streams.pop().unwrap();
                            let name_p1 = player_names.get(0).unwrap();
                            let msg_p1 = ServerMessage::START(name);
                            let msg_p2 = ServerMessage::START(name_p1.to_string());

                            stream
                                .try_clone()
                                .unwrap()
                                .write(serde_json::to_string(&msg_p2).unwrap().as_bytes())
                                .unwrap();

                            p1_stream
                                .write(serde_json::to_string(&msg_p1).unwrap().as_bytes())
                                .unwrap();
                            game_stats = GameStats {
                                p1_id: 1,
                                p2_id: 2,
                                last_winner_id: 0,
                                draw: false,
                                end_turn: false,
                                end_round: false,
                                p1_points: (0, 0),
                                p2_points: (0, 0),
                                gameover: false,
                                turn: 0,
                                played_cards: vec![Carta {
                                    naipe: Naipe::None,
                                    rank: Rank::None,
                                }],
                                last_played_card: manilha.clone(),
                            };
                        }
                    }
                    ClientMessage::GetCards => {
                        if streams.len() == 0 {
                            streams.push(stream.try_clone().unwrap());
                        } else {
                            baralho = cria_baralho();
                            manilha = baralho.pop().unwrap();

                            let minha_mao_p1 = Mao {
                                mao: vec![
                                    baralho.pop().unwrap(),
                                    baralho.pop().unwrap(),
                                    baralho.pop().unwrap(),
                                ],
                            };
                            let minha_mao_p2 = Mao {
                                mao: vec![
                                    baralho.pop().unwrap(),
                                    baralho.pop().unwrap(),
                                    baralho.pop().unwrap(),
                                ],
                            };
                            let p1 = Player {
                                is_turn: true,
                                id: 1,
                                cartas: minha_mao_p1,
                                manilha: manilha.clone(),
                                pode_trucar: true,
                            };
                            let p2 = Player {
                                is_turn: false,
                                id: 2,
                                cartas: minha_mao_p2,
                                manilha: manilha.clone(),
                                pode_trucar: true,
                            };

                            players.push(p1.clone());

                            players.push(p2.clone());

                            let msg_p1 = ServerMessage::SendPlayerCards(p1);
                            let msg_p2 = ServerMessage::SendPlayerCards(p2);
                            stream
                                .try_clone()
                                .unwrap()
                                .write(serde_json::to_string(&msg_p1).unwrap().as_bytes())
                                .unwrap();

                            let mut stream2 = streams.pop().unwrap();

                            stream2
                                .write(serde_json::to_string(&msg_p2).unwrap().as_bytes())
                                .unwrap();
                        }
                    }
                    ClientMessage::PlayCard(played) => {
                        let (carta, id) = played.clone();
                        let mut stream_opponent = streams.pop().unwrap();
                        let msg;
                        let msg_opponent;

                        if game_stats.end_round {
                            game_stats.end_round = false;
                            round_value = 1;
                            game_stats.last_winner_id = 0;
                            game_stats.draw = false;
                            game_stats.turn = 0;
                            game_stats.p1_points.0 = 0;
                            game_stats.p2_points.0 = 0;
                        }

                        if game_stats.end_turn {
                            //Aqui há uma carta na mesa um player está esperando para saber o resultado
                            let (op_carta, op_id): (Carta, u16) = played_cards.pop().unwrap();

                            game_stats.played_cards = vec![op_carta.clone(), carta.clone()];

                            let last_turn_draw = game_stats.draw;

                            game_stats.turn += 1;
                            game_stats.end_turn = false;
                            game_stats.last_played_card = carta.clone();

                            //Pega o id do vencedor deste turno
                            game_stats.last_winner_id = if carta.rank == manilha.rank.next()
                                && op_carta.rank == manilha.rank.next()
                            {
                                if op_carta.naipe < carta.naipe {
                                    id
                                } else {
                                    op_id
                                }
                            } else if carta.rank == manilha.rank.next() {
                                id
                            } else if op_carta.rank == manilha.rank.next() {
                                op_id
                            } else if op_carta.rank < carta.rank {
                                id
                            } else if op_carta.rank == carta.rank && last_turn_draw {
                                if op_carta.naipe < carta.naipe {
                                    id
                                } else {
                                    op_id
                                }
                            } else if op_carta.rank == carta.rank && !last_turn_draw {
                                game_stats.draw = true;
                                400
                            } else {
                                op_id
                            };

                            //Atribui a pontuação do turno para o vencedor se houver
                            if game_stats.last_winner_id == game_stats.p1_id {
                                game_stats.p1_points.0 += if last_turn_draw { 2 } else { 1 };
                            } else if game_stats.draw {
                                if game_stats.p1_points.0 > 0 {
                                    game_stats.p1_points.0 += if last_turn_draw {
                                        game_stats.last_winner_id = game_stats.p1_id;
                                        2
                                    } else {
                                        1
                                    };
                                } else if game_stats.p2_points.0 > 0 {
                                    game_stats.p2_points.0 += if last_turn_draw {
                                        game_stats.last_winner_id = game_stats.p2_id;
                                        2
                                    } else {
                                        1
                                    };
                                }
                                if game_stats.last_winner_id == game_stats.p1_id {
                                    game_stats.p1_points.0 += 2;
                                } else if game_stats.last_winner_id == game_stats.p2_id {
                                    game_stats.p2_points.0 += 2;
                                }
                            } else {
                                game_stats.p2_points.0 += if last_turn_draw { 2 } else { 1 };
                            }

                            //Verifica se alguém venceu o round
                            if game_stats.p1_points.0 >= 2 {
                                game_stats.p1_points.1 += round_value;
                                game_stats.end_round = true;
                            } else if game_stats.p2_points.0 >= 2 {
                                game_stats.p2_points.1 += round_value;
                                game_stats.end_round = true;
                            }

                            if game_stats.p1_points.1 >= 12 {
                                msg = ServerMessage::GameOver(game_stats.p1_id, game_stats.clone());
                                msg_opponent =
                                    ServerMessage::GameOver(game_stats.p1_id, game_stats.clone());
                            } else if game_stats.p2_points.1 >= 12 {
                                msg = ServerMessage::GameOver(game_stats.p2_id, game_stats.clone());
                                msg_opponent =
                                    ServerMessage::GameOver(game_stats.p2_id, game_stats.clone());
                            } else {
                                msg = ServerMessage::Update(game_stats.clone());
                                msg_opponent = ServerMessage::Update(game_stats.clone());
                                played_cards.clear();
                                game_stats.played_cards.clear();
                                game_stats.last_played_card = Carta {
                                    naipe: Naipe::None,
                                    rank: Rank::None,
                                }
                            }
                        } else {
                            //Aqui não há nenhuma carta na mesa
                            played_cards.push((carta.clone(), id));

                            game_stats.last_played_card = carta.clone();

                            game_stats.end_turn = true;

                            msg = ServerMessage::Wait;
                            msg_opponent = ServerMessage::PlayedCard(carta);
                        }

                        stream
                            .try_clone()
                            .unwrap()
                            .write(serialize(msg).as_bytes())
                            .unwrap();
                        stream_opponent
                            .write(serialize(msg_opponent).as_bytes())
                            .unwrap();
                    }
                    ClientMessage::Truco(truco_state) => {
                        let mut trucado_stream = streams.pop().unwrap();

                        truco_state_list.push((truco_state, stream));

                        trucado_stream
                            .write(serialize(ServerMessage::TrucoRequest).as_bytes())
                            .unwrap();
                    }
                    ClientMessage::TrucoResponse(resposta) => {
                        let (state, mut truqueiro_stream) = truco_state_list.pop().unwrap();

                        if resposta.0 == 0 {
                            round_value += if round_value == 1 { 2 } else { 3 };
                            truqueiro_stream.write(serialize(state).as_bytes()).unwrap();
                            streams.push(stream);
                        } else {
                            if resposta.1 == game_stats.p1_id {
                                game_stats.p2_points.1 += round_value;
                                game_stats.p2_points.0 = 0;
                                game_stats.p1_points.0 = 0;
                            } else {
                                game_stats.p1_points.1 += round_value;
                                game_stats.p1_points.0 = 0;
                                game_stats.p2_points.0 = 0;
                            }

                            truqueiro_stream
                                .write(
                                    serialize(ServerMessage::TrucoResponse(game_stats.turn as u8))
                                        .as_bytes(),
                                )
                                .unwrap();
                            stream
                                .try_clone()
                                .unwrap()
                                .write(
                                    serialize(ServerMessage::TrucoResponse(game_stats.turn as u8))
                                        .as_bytes(),
                                )
                                .unwrap();
                        }
                    }
                    //Estes dois protocolos fazem a mesma coisa
                    //Todo: Unir os dois em um só
                    ClientMessage::GetResponse => {
                        streams.push(stream.try_clone().unwrap());
                    }
                }
            }
            Err(_) => {
                panic!();
            }
        }
    }
}
