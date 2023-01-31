/// 启动工具
use log::info;
use tokio::signal;

pub fn print_banner() {
    let banner = r#"
###############################################WEEEEKj EEEEEK#########
######WGGDDDDDGGW################WWGGDDDDDGGW##GKEEEK. EEEEEEW########
##WGEKEEEEK.EEEEEEEEEEEEEEEEEKEEEEEEEEEEEEEEEEEEEEEEt   EEEEEEDW######
##WDKEEEEEL .EEEEEEEEEEEEEEEEEEEEEEEEK. KEEEEEEKEEEE    ,EEEEEEKEDWW##
##GEKEK                                    .EEEEE.         LEEEEEEDEGW
##WEEEEEE.   .K;fEKEKD   GKKf,DEEEEEEKi.E  GEj.tEEEKK  jKEKL iEKEEEKEG
##WEEEEEE.   .K                 E.          K:                  DEEEKG
###EKEEEE.   .K    DEEEEEEK:   .KD  :;      .;    KEG   KEi    DEEEEED
###EEEEEE.   .K           t:   .KE  .;  .EtjE:    ,       ,    EEEEEEG
###DEEEEE.   .K    EKKKEKKW:   .KE  .;  .t  t,    KE     Ki    EEEEEEG
##WKEEEEE    GE   :;;.   .;    :KE  .;  .t  i,    EEK. DEEt    EEEEKKW
#WGKEEEEE    EE  LKEK;   :K,..jEEE  .;  .t  D:                 DEEEEKW
#WDKEEEEE    EKEj. EK;   :Ki .tKKD  :;  .j  K,   jKEK. DKEE    DEEEEKW
WGKEEEEK.  :L        j   i          K,  .KEEK;.DEKKD    .KKKK,.DEEEEDW
WDDEEEK  :KK:  LEEEEK;   :EEEKEi   KK,   KEEEEEEEt        :DEEEEEEEEG#
WDDEK.EKEEEEEEEEE:;EK,   EEE:iEEEEEEK:  DEEEKEEEEEEE    :EEEEKEEDEDGW#
WGDDEEEEEEEEEEEEEEEEK..EEEEEEEEEEEEEK. EEEEEEEEEEEEE,   EEEEEEEDWW####
#WWGDDEEEEEEKKKKEEEEEEEEEEEEKDKDEKEKEEEEEEKEDGGKEEEEE  :EEEEEDW#######
##############WWGDEDKEEEDKEDGGWWGDDEKKDKEEDDW##EEEEEK  DEEEEKW########
###############################################WDEEEKi EEEEEK#########
###############################################WKEEEEE.EEEEDD#########
################################################DKDEED:KEEDEG#########
"#;
    println!("{}", banner)
}

/// 优雅关闭
pub async fn shutdown_signal_handler() {
    let ctrl_c = async {
        signal::ctrl_c().await.expect("Ctrl+C的信号监听器启动失败");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("程序关闭的信号监听器启动失败")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {
            println!();
            info!("收到Ctrl+C, 开始收尾");
        },
        _ = terminate => {
            info!("收到程序关闭信号，开始收尾");
        },
    }

    info!("收尾结束，程序关闭");
}
