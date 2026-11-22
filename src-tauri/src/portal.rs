use crate::model::{PipeWireRemote, PipeWireSource, Source};
use ashpd::desktop::{
    screencast::{CursorMode, Screencast, SourceType},
    PersistMode, Session,
};
use std::{future::Future, os::fd::OwnedFd, pin::Pin, sync::Arc};

#[derive(Debug)]
struct PortalSession {
    proxy: Screencast<'static>,
    session: Option<Session<'static, Screencast<'static>>>,
    runtime: tokio::runtime::Handle,
}

impl PipeWireRemote for PortalSession {
    fn open(&self) -> Pin<Box<dyn Future<Output = Result<OwnedFd, String>> + Send + '_>> {
        Box::pin(async move {
            let session = self
                .session
                .as_ref()
                .ok_or("A sessão de captura foi encerrada")?;
            self.proxy
                .open_pipe_wire_remote(session)
                .await
                .map_err(|e| format!("Não foi possível abrir o PipeWire: {e}"))
        })
    }
}

impl Drop for PortalSession {
    fn drop(&mut self) {
        if let Some(session) = self.session.take() {
            self.runtime.spawn(async move {
                if let Err(error) = session.close().await {
                    eprintln!("Ao encerrar o portal de captura: {error}");
                }
            });
        }
    }
}

/// The desktop compositor owns the picker; no capture permission is persisted.
pub async fn select() -> Result<Source, String> {
    let proxy = Screencast::new().await.map_err(|e| format!("Portal de captura indisponível. Instale xdg-desktop-portal e o backend do seu desktop: {e}"))?;
    let session = proxy.create_session().await.map_err(|e| e.to_string())?;
    let guard = PortalSession {
        proxy,
        session: Some(session),
        runtime: tokio::runtime::Handle::current(),
    };
    let session = guard
        .session
        .as_ref()
        .ok_or("Sessão do portal indisponível")?;
    let types = guard
        .proxy
        .available_source_types()
        .await
        .map_err(|e| e.to_string())?
        & (SourceType::Monitor | SourceType::Window);
    if types.is_empty() {
        return Err("O portal do seu desktop não oferece captura de monitor ou janela".into());
    }
    let modes = guard
        .proxy
        .available_cursor_modes()
        .await
        .unwrap_or(CursorMode::Hidden.into());
    let cursor = if modes.contains(CursorMode::Embedded) {
        CursorMode::Embedded
    } else if modes.contains(CursorMode::Hidden) {
        CursorMode::Hidden
    } else if modes.contains(CursorMode::Metadata) {
        CursorMode::Metadata
    } else {
        return Err("O portal não oferece um modo de cursor compatível".into());
    };
    guard
        .proxy
        .select_sources(session, cursor, types, false, None, PersistMode::DoNot)
        .await
        .map_err(|e| format!("Não foi possível selecionar a captura: {e}"))?
        .response()
        .map_err(|e| e.to_string())?;
    let response = guard
        .proxy
        .start(session, None)
        .await
        .map_err(|e| format!("A seleção de tela não foi concluída: {e}"))?
        .response()
        .map_err(|e| format!("Captura cancelada ou recusada pelo sistema: {e}"))?;
    let stream = response
        .streams()
        .first()
        .ok_or("O sistema não disponibilizou uma tela para captura")?;
    let (width, height) = stream.size().unwrap_or((0, 0));
    Ok(Source {
        id: "portal".into(),
        name: "Tela ou janela autorizada pelo sistema".into(),
        xid: 0,
        x: 0,
        y: 0,
        width: width.max(0) as u32,
        height: height.max(0) as u32,
        pipewire: Some(PipeWireSource {
            node_id: stream.pipe_wire_node_id(),
            remote: Arc::new(guard),
        }),
    })
}

pub async fn open_remote(source: &Source) -> Result<Option<OwnedFd>, String> {
    match &source.pipewire {
        Some(pipewire) => pipewire.remote.open().await.map(Some),
        None => Ok(None),
    }
}
