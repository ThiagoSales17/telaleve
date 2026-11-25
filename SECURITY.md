# Política de segurança

## Uso seguro

TelaLeve foi projetado para uso em uma rede local confiável. A sinalização HTTP/WebSocket não usa TLS e o serviço local pode escutar em todas as interfaces IPv4. Não exponha a porta do serviço diretamente à internet; limite o acesso com o firewall e evite redes locais não confiáveis. O vídeo usa WebRTC.

O pareamento exige aprovação no desktop e usa um código temporário, mas isso não substitui controles de rede nem uma auditoria de segurança.

## Reportar uma vulnerabilidade

Não publique detalhes de uma vulnerabilidade não corrigida em uma issue pública. Use o recurso **Report a vulnerability** na aba **Security** deste repositório, se estiver habilitado, para enviar um reporte privado. Inclua passos para reproduzir, impacto observado, versão/commit afetado e uma mitigação possível, sem anexar dados pessoais ou credenciais.

Se o reporte privado não estiver disponível, contate o mantenedor por um canal privado antes de divulgar os detalhes. Não há prazo de resposta ou SLA de correção garantido.
