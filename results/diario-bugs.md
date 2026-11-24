# Diário de bugs por plataforma

Formato: data, candidato, SO, o que aconteceu, causa, como contornou, tempo gasto (agente / revisão humana, estimado).

| Data | Candidato | SO | Problema | Causa | Contorno | Tempo |
| --- | --- | --- | --- | --- | --- | --- |
| 2026-11-24 | Todos | Windows 11 | `cargo build` bloqueado: "Uma política de Controle de Aplicativo bloqueou este arquivo" | Smart App Control ativo; binários compilados localmente não têm assinatura nem reputação | Gian desligou o SAC (permanente). Registrar o estado do SAC em cada execução. | agente ~10 min, Gian ~2 min |
| 2026-11-24 | Electron (ProseMirror) | Windows 11 | Texto digitado desfeito sem erro visível; só um `RangeError` no console do renderer | `prosemirror-view` 1.42.8 instala o próprio `prosemirror-model` 1.25.13; com o 1.25.4 fixado no `package.json` o bundle carregou duas cópias ("multiple versions of prosemirror-model") | Fixar `prosemirror-model` na mesma versão que o `prosemirror-view` resolve (1.25.13); verificar com `npm ls` | agente ~25 min |
| 2026-11-24 | Electron (ProseMirror) | Windows 11 | Nós `tag` (chips) sumiam ao reler o DOM | Nó sem `parseDOM`: o PM não consegue reconstruir o átomo a partir do DOM e perde as tags | Adicionado `parseDOM` com atributos `data-tag-*` no `toDOM` | agente ~10 min |
