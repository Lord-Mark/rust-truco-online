use serde::{Deserialize, Serialize};
use serde_json;
use std::io::{self, prelude::*};
use std::net::TcpStream;

//Definir as mensagens do servidor aqui

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
    GetResponse,
    PlayCard((Carta, u16)),
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
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Mao {
    pub mao: Vec<Carta>,
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

fn process_server_message(stream: &TcpStream) -> ServerMessage {
    let mut buffer = [0; 1024];
    let bytes_read = stream.try_clone().unwrap().read(&mut buffer).unwrap();
    let received_msg = serde_json::from_slice(&buffer[..bytes_read]).unwrap();
    received_msg
}
fn main() {
    let meu_ip: &str = "127.0.0.1:7878";
    let serialize = |msg| serde_json::to_string(&msg).unwrap();
    let mut stream = TcpStream::connect(meu_ip).unwrap();

    let mut username = String::new();

    let mut is_registered: bool = false;
    let mut my_id: u16 = 500;
    let mut my_stats: Player = Player {
        is_turn: false,
        id: 500,
        cartas: Mao {
            mao: vec![Carta {
                naipe: Naipe::None,
                rank: Rank::None,
            }],
        },
        manilha: Carta {
            naipe: Naipe::None,
            rank: Rank::None,
        },
        pode_trucar: false,
    };

    let mut op_name = String::new();

    println!("Digite seu nome:");
    let stdin = io::stdin();
    stdin.read_line(&mut username).unwrap();
    username = username.trim().to_string();

    let connect_message = serialize(ClientMessage::Connect(username.clone()));

    stream.write(connect_message.as_bytes()).unwrap();

    println!("Encontrando um oponente digno...");

    let mut msg: String;
    loop {
        match process_server_message(&stream) {
            ServerMessage::START(opponent_name) => {
                op_name = opponent_name;
                msg = serialize(ClientMessage::GetCards);
                println!("\n{} desafiou você!\n", op_name);
            }
            ServerMessage::SendPlayerCards(player) => {
                my_stats = player.clone();
                if !is_registered {
                    my_id = my_stats.id;
                }
                is_registered = true;
                if my_stats.is_turn {
                    let opt = my_turn(
                        &mut my_stats.cartas,
                        &my_stats.manilha,
                        None,
                        &my_stats.pode_trucar,
                    );
                    match opt {
                        None => {
                            let mut stats = player.clone();
                            stats.pode_trucar = false;
                            msg = serialize(ClientMessage::Truco(ServerMessage::SendPlayerCards(
                                stats,
                            )));
                        }
                        Some(carta) => {
                            msg = serialize(ClientMessage::PlayCard((carta, my_id)));
                            my_stats.is_turn = false;
                        }
                    }
                    println!("\nBoa jogada!, agora aguarde sua vez..\n");
                } else {
                    let cartas = &my_stats.cartas;
                    let manilha = &my_stats.manilha;

                    println!("===========================");
                    println!("Você recebeu suas cartas:\n");

                    for carta in &cartas.mao {
                        println!("\t{:?} de {:?}", carta.rank, carta.naipe);
                    }

                    println!("\nManilha: {:?}", manilha.rank.next());
                    println!("===========================");
                    println!("Aguarde sua vez...");

                    msg = serialize(ClientMessage::GetResponse);
                }
            }
            ServerMessage::Wait => {
                msg = serialize(ClientMessage::GetResponse);
                my_stats.is_turn = false;
            }

            ServerMessage::PlayedCard(carta) => {
                let opt = my_turn(
                    &mut my_stats.cartas,
                    &my_stats.manilha,
                    Some(&carta),
                    &my_stats.pode_trucar,
                );
                match opt {
                    None => {
                        my_stats.pode_trucar = false;
                        msg = serialize(ClientMessage::Truco(ServerMessage::PlayedCard(carta)));
                    }
                    Some(carta) => {
                        msg = serialize(ClientMessage::PlayCard((carta, my_id)));
                        my_stats.is_turn = true;
                    }
                }
            }

            ServerMessage::TrucoRequest => {
                let escolha: u8;
                println!("\n===========================");
                println!("ATENÇÃO, VOCÊ FOI TRUCADO!!");
                println!("===========================\n");
                println!("{op_name}: TRUUUUUUCO PAPUDO!!!");

                println!("\nVai aceitar??");
                println!("\t0: ENTÃO ME MOSTRA O QUE TU TEM (aceitar)");
                println!("\t1: Recusar (você perde o round atual)");
                // println!("\t2: SEEEIS, LADRÃÃO (aumentar a aposta)");

                let mut str = String::new();
                loop {
                    println!("\nSua escolha:");
                    io::stdin().read_line(&mut str).expect("Falhou em ler");

                    escolha = match str.trim().parse() {
                        Ok(num) => {
                            if num < 2 {
                                num
                            } else {
                                println!("\n\nDigite uma opção válida!\n");
                                continue;
                            }
                        }
                        Err(_) => continue,
                    };
                    break;
                }
                msg = serialize(ClientMessage::TrucoResponse((escolha, my_id)));
            }

            ServerMessage::Update(game_stats) => {
                println!("");
                if game_stats.last_winner_id == my_id {
                    println!("Você venceu este turno!");
                } else if game_stats.last_winner_id == 400 {
                    println!("Este turno empatou");
                } else {
                    println!("\n\nVocê perdeu este turno...");
                }
                println!("Cartas jogadas: ");
                for carta in game_stats.clone().played_cards {
                    if carta.rank == Rank::None {
                        println!("\tCarta escondida");
                    } else {
                        println!("\t{:?} de {:?}", carta.rank, carta.naipe);
                    }
                }

                if game_stats.end_round {
                    println!("Este round terminou!");
                }

                let (my_points, op_points) = if game_stats.p1_id == my_id {
                    (game_stats.p1_points, game_stats.p2_points)
                } else {
                    (game_stats.p2_points, game_stats.p1_points)
                };

                println!("{username}, esta é sua pontuação:");
                println!("\tRound {}", my_points.0);
                println!("\tTotal {}\n", my_points.1);

                println!("pontuação de {}", op_name);
                println!("\tRound {}", op_points.0);
                println!("\tTotal {}", op_points.1);

                if game_stats.end_round {
                    println!("\n============ Fim do round ============\n");
                    my_stats.pode_trucar = true;
                    if game_stats.turn == 2 {
                        //Se o jogo acabou em 2 rounds o turno é invertido
                        my_stats.is_turn = if my_stats.is_turn { false } else { true };
                    }
                    msg = serialize(ClientMessage::GetCards);
                } else {
                    if my_stats.is_turn {
                        let opt = my_turn(
                            &mut my_stats.cartas,
                            &my_stats.manilha,
                            None,
                            &my_stats.pode_trucar,
                        );
                        match opt {
                            None => {
                                my_stats.pode_trucar = false;
                                msg = serialize(ClientMessage::Truco(ServerMessage::Update(
                                    game_stats.clone(),
                                )));
                            }
                            Some(carta) => {
                                msg = serialize(ClientMessage::PlayCard((carta, my_id)));
                                my_stats.is_turn = false;
                            }
                        }
                    } else {
                        println!("\nVez do oponente começar o turno");
                        println!("Aguarde sua vez...");
                        msg = serialize(ClientMessage::GetResponse);
                    }
                }
                // my_stats.is_turn = if my_stats.is_turn { false } else { true };
                // panic!("Finalizando códiguin porque não tá pronto sabagaça");
            }
            ServerMessage::TrucoResponse(turno) => {
                if turno != 2 {
                    my_stats.is_turn = if my_stats.is_turn { false } else { true };
                }
                msg = serialize(ClientMessage::GetCards);
            }
            ServerMessage::GameOver(winner_id, game_stats) => {
                if winner_id == my_id {
                    println!("\n\n\nParabéns, você venceu o jogo!!!!");
                    println!("\n{username}: {}", game_stats.p1_points.1);
                    println!("\n{op_name}: {}", game_stats.p2_points.1);
                } else {
                    println!("\n\n\nQue pena, você perdeu o jogo....");
                    println!("\n{username}: {}", game_stats.p2_points.1);
                    println!("\n{op_name}: {}", game_stats.p1_points.1);
                }
                break;
            }
        }

        stream = TcpStream::connect(meu_ip).unwrap();
        stream.write(msg.as_bytes()).unwrap();
    }
}

fn my_turn(
    cartas: &mut Mao,
    manilha: &Carta,
    opponnet_card: Option<&Carta>,
    is_trucavel: &bool,
) -> Option<Carta> {
    let escolha: usize;

    loop {
        println!("\n----------------------------------");
        println!("Sua vez de jogar");
        if *is_trucavel {
            println!("\tOpção 0: Pedir truco");
        }
        println!("\tOpção 1: Esconder carta");
        for i in 0..(cartas.mao.len()) {
            println!(
                "\tOpção {}: {:?} de {:?}",
                i + 2,
                cartas.mao[i].rank,
                cartas.mao[i].naipe
            );
        }

        println!("Manilha: {:?}", manilha.rank.next());

        match opponnet_card {
            Some(op_card) => {
                println!("----------------------------------");
                if op_card.rank == Rank::None {
                    println!("O oponente escondeu a carta");
                } else {
                    println!(
                        "Carta do oponente: {:?} de {:?}",
                        op_card.rank, op_card.naipe
                    );
                }
            }
            _ => {}
        };
        println!("\nSua escolha: ");
        let mut str = String::new();
        io::stdin().read_line(&mut str).expect("Falhou em ler");

        escolha = match str.trim().parse() {
            Ok(num) => {
                if (num as usize) <= (cartas.mao.len() + 2) {
                    if !is_trucavel && num == 0 {
                        println!("Você não pode mais trucar neste round");
                        continue;
                    }
                    num
                } else {
                    println!("\n\n\nDigite uma opção válida!\n");
                    continue;
                }
            }
            Err(_) => continue,
        };
        break;
    }
    if escolha == 0 {
        None
    } else if escolha == 1 {
        let excluida: usize;
        loop {
            println!("\nCartas:");
            for i in 0..(cartas.mao.len()) {
                println!(
                    "\tOpção {}: {:?} de {:?}",
                    i, cartas.mao[i].rank, cartas.mao[i].naipe
                );
            }
            println!("\nEscolha a carta que deseja esconder: ");
            let mut str = String::new();
            io::stdin().read_line(&mut str).expect("Falhou em ler");

            excluida = match str.trim().parse() {
                Ok(num) => {
                    if (num as usize) <= (cartas.mao.len()) {
                        num
                    } else {
                        println!("\n\n\nDigite uma opção válida!\n");
                        continue;
                    }
                }
                Err(_) => continue,
            };
            break;
        }
        cartas.mao.remove(excluida);

        Some(Carta {
            naipe: Naipe::None,
            rank: Rank::None,
        })
    } else {
        Some(cartas.mao.remove(escolha - 2))
    }
}
