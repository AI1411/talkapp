use chrono::Utc;
use dotenv::dotenv;
use sea_orm::{DatabaseConnection, DbErr, EntityTrait, Set};
use sea_orm::ActiveValue::NotSet;
use talkapp::domain::entity::{users, post, messages, reactions, reaction_types};
use talkapp::domain::entity::groups;
use talkapp::domain::entity::group_members;
use talkapp::domain::entity::group_messages;

async fn seed_data() -> Result<(), DbErr> {
    dotenv().ok();
    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let db = sea_orm::Database::connect(&database_url).await?;

    println!("データベースに接続しました");

    // ダミーデータの投入を実行
    insert_users(&db).await?;
    println!("ユーザーデータを投入しました");

    let user_ids = fetch_user_ids(&db).await?;
    println!("ユーザーID: {:?}", user_ids);

    insert_posts(&db, &user_ids).await?;
    println!("投稿データを投入しました");

    insert_messages(&db, &user_ids).await?;
    println!("メッセージデータを投入しました");

    let message_ids = fetch_message_ids(&db).await?;
    println!("メッセージID: {:?}", message_ids);

    insert_groups(&db, &user_ids).await?;
    println!("グループデータを投入しました");

    let group_ids = fetch_group_ids(&db).await?;
    println!("グループID: {:?}", group_ids);

    insert_group_members(&db, &group_ids, &user_ids).await?;
    println!("グループメンバーデータを投入しました");

    insert_group_messages(&db, &group_ids, &user_ids).await?;
    println!("グループメッセージデータを投入しました");

    insert_reactions(&db, &user_ids, &message_ids).await?;
    println!("リアクションデータを投入しました");

    println!("すべてのダミーデータの投入が完了しました");

    Ok(())
}

async fn insert_users(db: &DatabaseConnection) -> Result<(), DbErr> {
    let now = Utc::now().naive_utc();

    // ユーザーデータの配列
    let users_data = [
        (
            "田中太郎",
            "tanaka@example.com",
            Some("エンジニア。Rustとバックエンド開発が得意です。"),
            Some(28),
            Some("男性"),
            Some("東京都")
        ),
        (
            "佐藤花子",
            "sato@example.com",
            Some("デザイナー。UIデザインとフロントエンド開発ができます。"),
            Some(26),
            Some("女性"),
            Some("神奈川県")
        ),
        (
            "鈴木一郎",
            "suzuki@example.com",
            Some("プロジェクトマネージャー。チームマネジメントが得意です。"),
            Some(35),
            Some("男性"),
            Some("大阪府")
        ),
        (
            "高橋直子",
            "takahashi@example.com",
            Some("マーケター。デジタルマーケティングを担当しています。"),
            Some(31),
            Some("女性"),
            Some("福岡県")
        ),
        (
            "伊藤健太",
            "ito@example.com",
            Some("バックエンドエンジニア。データベース設計が得意です。"),
            Some(29),
            Some("男性"),
            Some("北海道")
        ),
    ];

    // ユーザーデータを登録
    for (name, email, description, age, gender, address) in users_data {
        let user = users::ActiveModel {
            id: NotSet,
            name: Set(name.to_string()),
            email: Set(email.to_string()),
            description: Set(description.map(|s| s.to_string())),
            age: Set(age),
            gender: Set(gender.map(|s| s.to_string())),
            address: Set(address.map(|s| s.to_string())),
            created_at: Set(now),
            updated_at: Set(now),
            deleted_at: NotSet,
        };

        users::Entity::insert(user).exec(db).await?;
    }

    Ok(())
}

async fn fetch_user_ids(db: &DatabaseConnection) -> Result<Vec<i32>, DbErr> {
    let users = users::Entity::find().all(db).await?;
    let user_ids = users.into_iter().map(|u| u.id).collect();
    Ok(user_ids)
}

async fn insert_posts(db: &DatabaseConnection, user_ids: &[i32]) -> Result<(), DbErr> {
    let posts_data = [
        (
            "Rustの非同期プログラミングについて考察してみました。tokioとasync-stdの両方を使ってみた感想です。",
            &user_ids[0]
        ),
        (
            "Unityを使って簡単なゲームを作ってみました。初心者でも意外と簡単にできましたよ！",
            &user_ids[1]
        ),
        (
            "リモートワークでのチームマネジメントのコツをまとめてみました。定期的なコミュニケーションが大事です。",
            &user_ids[2]
        ),
        (
            "最近のデジタルマーケティングのトレンドについて調査しました。データ分析の重要性が増しています。",
            &user_ids[3]
        ),
        (
            "PostgreSQLのパフォーマンスチューニングのベストプラクティスをまとめました。インデックス設計が重要です。",
            &user_ids[4]
        ),
        (
            "React Hooksを使った状態管理の実践的なパターンを紹介します。",
            &user_ids[1]
        ),
        (
            "Dockerを使った開発環境の構築方法を解説します。環境の統一が捗りますよ！",
            &user_ids[0]
        ),
        (
            "アジャイル開発とスクラムの違いについて整理してみました。",
            &user_ids[2]
        ),
        (
            "Pythonを使ったデータ分析の基本的な流れを解説します。pandasとmatplotlibを使います。",
            &user_ids[4]
        ),
        (
            "SNSマーケティングの効果測定方法について考察しました。",
            &user_ids[3]
        ),
    ];

    // 現在時刻をISO 8601形式の文字列で取得
    let now_str = Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();

    for (body, user_id) in posts_data {
        let post = post::ActiveModel {
            id: NotSet,
            body: Set(body.to_string()),
            user_id: Set(*user_id),
            created_at: Set(now_str.clone()),
        };

        post::Entity::insert(post).exec(db).await?;
    }

    Ok(())
}

async fn insert_messages(db: &DatabaseConnection, user_ids: &[i32]) -> Result<(), DbErr> {
    let now = Utc::now().naive_utc();

    // メッセージのペア（送信者ID, 受信者ID, 内容）
    let messages_data = [
        (user_ids[0], user_ids[1], "こんにちは、佐藤さん。プロジェクトの進捗はどうですか？"),
        (user_ids[1], user_ids[0], "田中さん、おはようございます。順調に進んでいます。"),
        (user_ids[0], user_ids[1], "よかった。何か困っていることはありますか？"),
        (user_ids[1], user_ids[0], "特にないです。デザインが終わったら共有します。"),
        (user_ids[2], user_ids[3], "高橋さん、資料の作成お願いできますか？"),
        (user_ids[3], user_ids[2], "鈴木さん、了解しました。いつまでに必要ですか？"),
        (user_ids[2], user_ids[3], "来週の金曜日までにお願いします。"),
        (user_ids[3], user_ids[2], "分かりました。それまでに準備します。"),
        (user_ids[4], user_ids[0], "田中さん、データベースの設計について相談があります。"),
        (user_ids[0], user_ids[4], "伊藤さん、いいですよ。どんな内容ですか？"),
        (user_ids[4], user_ids[0], "インデックスの設計で悩んでいます。時間あるときに見てもらえませんか？"),
        (user_ids[0], user_ids[4], "明日の午後なら時間あります。オンラインミーティングしましょう。"),
        (user_ids[1], user_ids[2], "鈴木さん、次のミーティングはいつですか？"),
        (user_ids[2], user_ids[1], "佐藤さん、来週の月曜日の10時からです。"),
        (user_ids[1], user_ids[3], "高橋さん、マーケティング資料を共有いただけますか？"),
        (user_ids[3], user_ids[1], "佐藤さん、もちろんです。すぐに送ります。"),
        (user_ids[3], user_ids[4], "伊藤さん、データ分析結果を教えてください。"),
        (user_ids[4], user_ids[3], "高橋さん、まだ分析中です。明日までにはお送りします。"),
        (user_ids[2], user_ids[0], "田中さん、コードレビューお願いできますか？"),
        (user_ids[0], user_ids[2], "鈴木さん、了解です。GitHubのURLを送ってください。"),
    ];

    for (sender_id, receiver_id, content) in messages_data {
        let message = messages::ActiveModel {
            id: NotSet,
            sender_id: Set(sender_id),
            receiver_id: Set(receiver_id),
            content: Set(content.to_string()),
            is_read: Set(false), // 未読状態で作成
            created_at: Set(now),
            updated_at: Set(now),
            deleted_at: NotSet,
        };

        messages::Entity::insert(message).exec(db).await?;
    }

    Ok(())
}

async fn fetch_message_ids(db: &DatabaseConnection) -> Result<Vec<i32>, DbErr> {
    let messages_list = messages::Entity::find().all(db).await?;
    let message_ids = messages_list.into_iter().map(|m| m.id).collect();
    Ok(message_ids)
}

async fn insert_groups(db: &DatabaseConnection, user_ids: &[i32]) -> Result<(), DbErr> {
    let now = Utc::now().naive_utc();

    // グループデータ（名前, 説明, 作成者ID）
    let groups_data = [
        (
            "バックエンドチーム",
            Some("バックエンド開発に関する議論を行うグループです。"),
            user_ids[0] // 田中太郎がバックエンドチームの作成者
        ),
        (
            "フロントエンドチーム",
            Some("フロントエンド開発に関する議論を行うグループです。"),
            user_ids[1] // 佐藤花子がフロントエンドチームの作成者
        ),
        (
            "マネジメントチーム",
            Some("プロジェクト管理に関する議論を行うグループです。"),
            user_ids[2] // 鈴木一郎がマネジメントチームの作成者
        ),
        (
            "マーケティングチーム",
            Some("マーケティング戦略に関する議論を行うグループです。"),
            user_ids[3] // 高橋直子がマーケティングチームの作成者
        ),
        (
            "全社共有",
            Some("全社員向けの情報共有を行うグループです。"),
            user_ids[2] // 鈴木一郎が全社共有グループの作成者
        ),
    ];

    for (name, description, creator_id) in groups_data {
        let group = groups::ActiveModel {
            id: NotSet,
            name: Set(name.to_string()),
            description: Set(description.map(|s| s.to_string())),
            creator_id: Set(creator_id),
            created_at: Set(now),
            updated_at: Set(now),
            deleted_at: NotSet,
        };
        groups::Entity::insert(group).exec(db).await?;
    }

    Ok(())
}

async fn fetch_group_ids(db: &DatabaseConnection) -> Result<Vec<i32>, DbErr> {
    let groups_list = groups::Entity::find().all(db).await?;
    let group_ids = groups_list.into_iter().map(|g| g.id).collect();
    Ok(group_ids)
}

async fn insert_group_members(db: &DatabaseConnection, group_ids: &[i32], user_ids: &[i32]) -> Result<(), DbErr> {
    let now = Utc::now().naive_utc();

    // グループメンバーデータを構築（グループID, ユーザーID, ロール）
    // 既に作成者は自動的にadminとして登録されているので、それ以外のメンバーを追加
    let group_members_data = [
        // バックエンドチーム（group_ids[0]）のメンバー
        (group_ids[0], user_ids[4], "member"), // 伊藤健太

        // フロントエンドチーム（group_ids[1]）のメンバー
        (group_ids[1], user_ids[0], "member"), // 田中太郎

        // マネジメントチーム（group_ids[2]）のメンバー
        (group_ids[2], user_ids[0], "member"), // 田中太郎
        (group_ids[2], user_ids[1], "member"), // 佐藤花子

        // マーケティングチーム（group_ids[3]）のメンバー
        (group_ids[3], user_ids[1], "member"), // 佐藤花子

        // 全社共有（group_ids[4]）のメンバー
        (group_ids[4], user_ids[0], "member"), // 田中太郎
        (group_ids[4], user_ids[1], "member"), // 佐藤花子
        (group_ids[4], user_ids[3], "member"), // 高橋直子
        (group_ids[4], user_ids[4], "member"), // 伊藤健太
    ];

    for (group_id, user_id, role) in group_members_data {
        let member = group_members::ActiveModel {
            id: NotSet,
            group_id: Set(group_id),
            user_id: Set(user_id),
            role: Set(role.to_string()),
            created_at: Set(now),
            updated_at: Set(now),
            deleted_at: NotSet,
        };

        group_members::Entity::insert(member).exec(db).await?;
    }

    Ok(())
}

async fn insert_group_messages(db: &DatabaseConnection, group_ids: &[i32], user_ids: &[i32]) -> Result<(), DbErr> {
    let now = Utc::now().naive_utc();

    // グループメッセージデータ（グループID, 送信者ID, メッセージ内容）
    let group_messages_data = [
        // バックエンドチーム（group_ids[0]）のメッセージ
        (
            group_ids[0],
            user_ids[0],
            "バックエンドチームのみなさん、こんにちは。今週の目標についてディスカッションしましょう。"
        ),
        (
            group_ids[0],
            user_ids[4],
            "了解です。データベースのパフォーマンス改善が最優先だと思います。"
        ),

        // フロントエンドチーム（group_ids[1]）のメッセージ
        (
            group_ids[1],
            user_ids[1],
            "新しいUIデザインのモックアップを共有します。フィードバックをお願いします。"
        ),
        (
            group_ids[1],
            user_ids[0],
            "素晴らしいデザインですね。レスポンシブ対応についても考慮されていますか？"
        ),

        // マネジメントチーム（group_ids[2]）のメッセージ
        (
            group_ids[2],
            user_ids[2],
            "次回のプロジェクトレビューは来週月曜日に行います。資料の準備をお願いします。"
        ),
        (
            group_ids[2],
            user_ids[1],
            "了解しました。デザインチームの進捗を報告します。"
        ),
        (
            group_ids[2],
            user_ids[0],
            "バックエンドの実装状況について報告書を作成します。"
        ),

        // マーケティングチーム（group_ids[3]）のメッセージ
        (
            group_ids[3],
            user_ids[3],
            "Q2のマーケティング戦略について議論しましょう。"
        ),
        (
            group_ids[3],
            user_ids[1],
            "ユーザーインタビューの結果を分析した資料を準備します。"
        ),

        // 全社共有（group_ids[4]）のメッセージ
        (
            group_ids[4],
            user_ids[2],
            "全社員の皆さん、四半期の業績報告会を来週金曜日に開催します。参加をお願いします。"
        ),
        (
            group_ids[4],
            user_ids[0],
            "技術部門からの報告を担当します。"
        ),
        (
            group_ids[4],
            user_ids[3],
            "マーケティング部門の成果についてプレゼンします。"
        ),
    ];

    for (group_id, sender_id, content) in group_messages_data {
        let message = group_messages::ActiveModel {
            id: NotSet,
            group_id: Set(group_id),
            sender_id: Set(sender_id),
            content: Set(content.to_string()),
            created_at: Set(now),
            updated_at: Set(now),
            deleted_at: NotSet,
        };

        group_messages::Entity::insert(message).exec(db).await?;
    }

    Ok(())
}

async fn insert_reactions(db: &DatabaseConnection, user_ids: &[i32], message_ids: &[i32]) -> Result<(), DbErr> {
    let now = Utc::now().naive_utc();

    // reaction_typesテーブルからリアクションタイプを取得
    let reaction_types_list = reaction_types::Entity::find().all(db).await?;

    if reaction_types_list.is_empty() {
        println!("リアクションタイプが見つかりません。マイグレーションを実行していることを確認してください。");
        return Ok(());
    }

    // 最初の5つのメッセージに対してリアクションを追加
    for i in 0..5 {
        if i < message_ids.len() {
            // 各メッセージに複数のリアクションを追加
            let message_id = message_ids[i];

            // 異なるユーザーが異なるリアクションを付ける
            for j in 0..3 {
                if j < user_ids.len() && j < reaction_types_list.len() {
                    let user_id = user_ids[j];
                    let reaction_type_id = reaction_types_list[j].id;

                    let reaction = reactions::ActiveModel {
                        id: NotSet,
                        user_id: Set(user_id),
                        message_id: Set(message_id),
                        reaction_type_id: Set(reaction_type_id),
                        created_at: Set(now),
                        updated_at: Set(now),
                        deleted_at: NotSet,
                    };

                    reactions::Entity::insert(reaction).exec(db).await?;
                }
            }
        }
    }

    Ok(())
}

#[tokio::main]
async fn main() {
    if let Err(err) = seed_data().await {
        eprintln!("エラーが発生しました: {}", err);
        std::process::exit(1);
    }
}