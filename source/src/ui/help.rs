//! Offline help written against the actions and limits of the native application.
pub struct Topic {
    pub title: &'static str,
    pub icon: &'static str,
    pub keywords: &'static str,
    pub intro: &'static str,
    pub steps: &'static [&'static str],
    pub tip: &'static str,
    pub category: Option<usize>,
}
pub const TOPICS: &[Topic] = &[
    Topic {
        title: "Sua primeira gravação",
        icon: "record",
        keywords: "inicio começar tutorial gravar captura video",
        intro: "Da escolha da fonte ao arquivo salvo, em quatro passos.",
        steps: &[
            "Abra Nova gravação. Escolha tela inteira, janela, região ou janela de jogo.",
            "Confira o monitor ou a janela. Ative o microfone e o som do sistema que deseja incluir.",
            "Escolha a pasta de vídeos, a resolução e o FPS. Inicie: a contagem aparece sobre o monitor da captura.",
            "Pause ou retome quando precisar. Pare para finalizar o MP4; ele aparece na Biblioteca. Aguarde o aviso de gravação salva antes de fechar.",
        ],
        tip: "O app pode se minimizar ao começar. Os atalhos globais continuam funcionando fora da janela.",
        category: None,
    },
    Topic {
        title: "Tela, janela, região e jogo",
        icon: "monitor",
        keywords: "fonte monitor selecionar area game janela preta",
        intro: "Escolha apenas o conteúdo que precisa aparecer no vídeo.",
        steps: &[
            "Tela inteira captura o monitor escolhido em Configurações › Captura.",
            "Janela captura a janela selecionada. Região abre uma seleção sobre a área de trabalho: arraste a área e confirme.",
            "Jogo usa a janela escolhida por Windows Graphics Capture. Prefira modo janela ou sem bordas; conteúdo protegido e alguns jogos podem impedir a captura.",
            "Ative a prévia para conferir o enquadramento. Ela preserva a proporção da fonte. O app pode ser ocultado da captura para evitar o efeito de espelho.",
        ],
        tip: "Se a janela de origem fechar, consulte Recuperação. Captura exclusiva por hook de jogo não está disponível.",
        category: Some(1),
    },
    Topic {
        title: "Microfone e som do sistema",
        icon: "audio",
        keywords: "audio mic som volume dispositivo mudo silencio ruido",
        intro: "As duas fontes podem ser usadas juntas ou separadamente.",
        steps: &[
            "Abra Configurações › Áudio e escolha o dispositivo de entrada e a saída do sistema.",
            "Ative apenas as fontes necessárias. Padrão do Windows acompanha o dispositivo padrão ao iniciar a captura.",
            "Os medidores mostram o sinal recebido durante a gravação. Faça uma amostra curta e reproduza antes de uma sessão importante.",
            "Em Nova gravação, escolha Vídeo com áudio, Vídeo sem áudio, Tela + áudio separado ou Somente áudio. No modo separado o MP4 é silencioso e o .audio.wav contém a mistura; .mic.wav e .system.wav preservam as fontes.",
        ],
        tip: "Ctrl+Shift+A abre o gravador de áudio. Ajuste ganhos de 0 a 200% e escolha sua pasta de áudio. PCM estéreo 48 kHz/16 bits sem compressão; RF64 entra automaticamente ao ultrapassar 4 GiB. Isso não certifica sessões de horas. F8/F9/F10 seguem os atalhos configurados. No editor, importe WAVs para o mix independente.",
        category: Some(2),
    },
    Topic {
        title: "Webcam, cursor e cliques",
        icon: "webcam",
        keywords: "camera webcam cursor mouse clique sobreposicao overlay",
        intro: "Inclua sua câmera e torne a interação na tela mais clara.",
        steps: &[
            "Conecte a câmera e atualize os dispositivos. Escolha a webcam em Configurações › Captura ou Nova gravação.",
            "Ative a câmera e ajuste posição e tamanho. A prévia depende da disponibilidade e das permissões do dispositivo.",
            "Mostrar cursor inclui o ponteiro. Destacar cliques adiciona um efeito visual opcional.",
            "Confira uma gravação curta. A webcam é composta no vídeo; ela não é salva como uma faixa de vídeo independente.",
        ],
        tip: "Se a câmera não aparecer, feche aplicativos que a estejam usando e confira as permissões de câmera do Windows.",
        category: Some(1),
    },
    Topic {
        title: "Pastas e organização",
        icon: "folder",
        keywords: "salvar destino armazenamento nome exportacao screenshot print cache espaço disco",
        intro: "Você escolhe onde cada tipo de arquivo fica no computador.",
        steps: &[
            "Em Armazenamento, escolha as pastas de vídeos, capturas de tela, cache, exportações e projetos.",
            "O seletor do Windows valida a escrita e cria a pasta se necessário. Unidades removíveis precisam estar conectadas.",
            "Use {data}, {hora} e {contador} na regra de nomeação. Organize por data, tipo ou projeto.",
            "Defina alertas de espaço e o limite da biblioteca. Excluir na Biblioteca envia o conjunto de arquivos conhecidos para a Lixeira, após confirmação.",
        ],
        tip: "Mudar a pasta padrão afeta novas gravações. Os arquivos existentes continuam no lugar original.",
        category: Some(5),
    },
    Topic {
        title: "Editar, dividir e cortar",
        icon: "edit",
        keywords: "editor edicao rapida tutorial timeline trim split excluir clipe desfazer refazer",
        intro: "Um fluxo direto de edição sem alterar seu vídeo original.",
        steps: &[
            "Na Biblioteca, selecione um vídeo e escolha Editar; ou importe mídia no editor. A timeline reúne os clipes em sequência.",
            "Clique na régua ou na barra da prévia para navegar. Espaço reproduz ou pausa; selecione o clipe desejado na timeline.",
            "Dividir cria dois clipes na posição do cursor. Cortar início remove o começo do clipe sob o cursor. Excluir remove o clipe selecionado; use divisão antes para remover só um trecho.",
            "Desfazer e Refazer recuperam alterações da sessão. Salvar projeto guarda uma receita .wyzproj; abrir projeto retoma a edição. Preserve os arquivos originais.",
        ],
        tip: "Editar durante a reprodução pausa o vídeo. Os botões sem mídia ficam desativados; um erro de corte perto da borda preserva o projeto.",
        category: None,
    },
    Topic {
        title: "Velocidade, volume e efeitos",
        icon: "sliders",
        keywords: "blur texto seta crop fade velocidade volume efeitos privacidade cortar tela",
        intro: "Ajustes simples, com resultado visível na prévia e na exportação.",
        steps: &[
            "Velocidade e volume atuam no clipe selecionado. Use os controles de Propriedades para ajustes contínuos; os botões alternam valores predefinidos.",
            "Texto adiciona uma legenda na posição atual por até cinco segundos. Ela fica salva no projeto; o painel lista as legendas existentes.",
            "Seta, blur central, crop e fade são alternadores. A seta usa uma posição fixa; o blur cobre uma área central; crop amplia a região central.",
            "Os efeitos são aplicados ao projeto. A interface mostra quais estão ativos. Para remover a última alteração, use Desfazer; exporte para gerar o resultado final.",
        ],
        tip: "Blur central não detecta dados sensíveis automaticamente. Confira todos os quadros relevantes antes de compartilhar. A velocidade também altera o tom do áudio.",
        category: None,
    },
    Topic {
        title: "Exportar e compartilhar",
        icon: "export",
        keywords: "render exportacao mp4 h264 qualidade resolucao fps bitrate progresso erro",
        intro: "Crie um MP4 novo mantendo o projeto e as fontes intactos.",
        steps: &[
            "Salve seu projeto. Escolha Exportar no editor ou na Biblioteca.",
            "Defina nome, pasta final, resolução, FPS e bitrate. MP4 com H.264/AAC é o formato disponível nesta versão.",
            "Inicie a exportação e acompanhe o progresso. Mantenha os arquivos de origem disponíveis até o final.",
            "Ao concluir, abra a pasta e reproduza o arquivo. Verifique enquadramento, início/fim, áudio e legendas.",
        ],
        tip: "Um arquivo .wyzproj contém referências à mídia; ele não é um vídeo exportado. MKV, WebM e codecs externos ainda não são oferecidos.",
        category: Some(3),
    },
    Topic {
        title: "Temas, ícones e modo foco",
        icon: "palette",
        keywords: "aparencia personalizar cores tema animacao fundo icone svg simplificado compacto foco",
        intro: "Ajuste seu ambiente sem reiniciar o aplicativo.",
        steps: &[
            "Abra Aparência. Escolha um dos oito temas ou Personalizado. Edite uma cor #RRGGBB ou use Paleta completa para alterar as quatro juntas, incluindo fundos claros; o contraste é validado.",
            "Os ícones acompanham o tema. Você pode escolher estilo, espessura e cor própria. Em Componentes › Ícones, substitua um vetor por SVG local e restaure quando quiser.",
            "Ative Fundo em movimento para um efeito discreto do tema. O movimento pausa durante gravação e exportação.",
            "O menu de interface no cabeçalho oferece Padrão, Foco, Compacto e Compacto · foco. A janela compacta guarda o tamanho da interface completa; o botão Expandir permite voltar.",
        ],
        tip: "Ocultar o app da captura evita repetição da prévia. Desative essa opção em Aparência quando quiser gravar o próprio WyzarCapture.",
        category: Some(6),
    },
    Topic {
        title: "Atalhos e teclado",
        icon: "keyboard",
        keywords: "teclas f8 f9 f10 atalho teclado acessibilidade esc tab espaco ctrl",
        intro: "Os atalhos de gravação abaixo refletem suas configurações atuais.",
        steps: &[
            "Os três atalhos de gravação são globais e funcionam com outro aplicativo em foco. Altere-os em Configurações › Atalhos; use teclas diferentes e observe avisos de conflito.",
            "Tab e Shift+Tab percorrem controles; Enter ou Espaço ativam o controle focado. Setas percorrem listas abertas. Escape fecha o seletor ou modal; no app, também cancela a contagem.",
            "No editor, sem campo em edição: Espaço reproduz, S divide, Delete remove, setas esquerda/direita navegam um segundo. Ctrl+S salva; Ctrl+Z desfaz; Ctrl+Shift+Z ou Ctrl+Y refaz; Ctrl+E abre exportação.",
            "Ctrl+Shift+A abre Somente áudio com a janela ativa. F1 abre a ajuda da tela. Em painéis com rolagem, Page Up/Down rolam; Home/End vão ao início/fim. Em campos, Ctrl+A seleciona tudo e Ctrl+V cola.",
        ],
        tip: "Se um controle estiver focado, Espaço o ativa. Clique na área vazia ou use Escape para liberar o foco antes dos atalhos do editor.",
        category: Some(4),
    },
    Topic {
        title: "Resolver problemas",
        icon: "warning",
        keywords: "erro falha lento travou preto sem som permissao encoder driver fps recuperacao",
        intro: "Diagnóstico local, com passos concretos e sem enviar seus dados.",
        steps: &[
            "Tela preta: confira a fonte, reabra a janela e tente modo sem bordas. Se estiver gravando o app, desative Ocultar WyzarCapture em Aparência.",
            "Sem áudio: confira as fontes, os dispositivos e o volume do Windows. Para microfone/câmera, confira também as permissões do Windows.",
            "Gravação lenta: teste 1080p30, reduza o bitrate e feche tarefas pesadas. Consulte Diagnóstico para as métricas da sessão e teste Perfis suportados antes de aumentar a qualidade.",
            "Falha ao salvar: confira espaço livre, conexão da unidade e pasta de destino. O detalhe do erro e os logs ficam disponíveis no diagnóstico; copie um resumo sem caminhos privados.",
        ],
        tip: "A resolução configurada e o FPS desejado não garantem desempenho sustentado. Valide seu cenário com uma gravação curta.",
        category: Some(7),
    },
    Topic {
        title: "Recuperação e privacidade",
        icon: "shield",
        keywords: "crash falha interrompida recuperar checkpoint segmento local offline privacidade",
        intro: "Se uma sessão interromper, preserve o material antes de retomar.",
        steps: &[
            "Com recuperação ativada, o aplicativo finaliza segmentos e grava checkpoints locais. Ao reabrir, sessões interrompidas aparecem em Recuperação.",
            "Abra os detalhes, escolha Recuperar e aguarde o novo vídeo na pasta de vídeos. O arquivo é reaberto antes de confirmar o resultado.",
            "O último segmento ainda aberto pode ser perdido. Não há garantia para falta de energia, falha física de disco ou conteúdo que não chegou a um checkpoint.",
            "Preferências, projetos, índices e logs ficam no computador. O app não exige conta online, não envia telemetria e não baixa componentes silenciosamente.",
        ],
        tip: "Projetos guardam caminhos para as fontes. Preserve vídeo e auxiliares juntos; use reimportação na Biblioteca se mover um arquivo.",
        category: Some(7),
    },
    Topic {
        title: "Gravar no modo compacto",
        icon: "compact",
        keywords: "compacto foco simplificado pequeno flutuante fixar pin tela gravar alternar interface atalhos",
        intro: "Um gravador flutuante com as mesmas fontes, temas e arquivos da interface completa.",
        steps: &[
            "No menu de interface do cabeçalho, escolha Modo compacto. Área define a fonte; o seletor inferior escolhe monitor, janela ou região. Sons controla microfone e sistema; Dispositivo escolhe e ativa a webcam.",
            "Escolha resolução e FPS e confira a pasta pelo botão de pasta. Gravar inicia com as preferências atuais e mostra a contagem sobre o monitor. Durante a sessão, Parar salva e Pausar permite retomar.",
            "O menu do próprio compacto oferece os quatro modos. Ctrl+Alt+1 volta ao padrão; Ctrl+Alt+2 abre Foco; Ctrl+Alt+3 abre Compacto e Ctrl+Alt+4 abre Compacto · foco. Esses atalhos de interface funcionam com a janela do app ativa.",
            "Use o alfinete para manter o compacto sobre outras janelas. Expandir volta à interface completa com seu tamanho anterior. A engrenagem abre ajustes completos temporariamente; fechar os ajustes retorna ao compacto.",
        ],
        tip: "A paleta também colore a marca e o ícone da janela/bandeja. O tema personalizado e suas substituições de ícones continuam ativos. Minimizar ao gravar e ocultar o app continuam configuráveis em Aparência.",
        category: None,
    },
    Topic {
        title: "Trilhas de áudio e qualidade",
        icon: "audio",
        keywords: "tracks independente mute solo ganho offset fade rf64 qualidade encoder hardware software desempenho",
        intro: "Controle microfone e sistema separadamente e confira o desempenho antes de uma gravação importante.",
        steps: &[
            "No editor, abra Trilhas ou Ctrl+Shift+T. Ao importar um vídeo com .mic.wav/.system.wav, as fontes são associadas automaticamente.",
            "Ative Mix independente. Mute silencia uma faixa; Solo isola as marcadas. Ganho varia de 0 a 200%; Offset move em passos de 50 ms; Fade aplica entrada/saída.",
            "Posicione o cursor da timeline dentro da faixa e use Cortar ou Dividir. Excluir remove o trecho do projeto; os arquivos originais são preservados. Ctrl+Z e Ctrl+Shift+Z desfazem/refazem.",
            "A prévia e a exportação usam o mix ativo, incluindo cortes, volume e fades. Projetos antigos mantêm áudio do clipe até a escolha do usuário. Em Formatos e dispositivos, Testar desempenho mede 5 s e sugere ajustes sem alterar resolução/FPS.",
        ],
        tip: "H.264 High usa NV12/BT.709 explícito; AAC estéreo fica em 192 kbps, máximo aceito pelo encoder Microsoft usado. WAV preserva PCM sem compressão. Dados inexistentes, câmeras ausentes e encoders recusados produzem aviso ou erro; enumeração não certifica FPS sustentado.",
        category: Some(3),
    },
    Topic {
        title: "Idioma do aplicativo e do instalador",
        icon: "settings",
        keywords: "idioma tradução inglês espanhol chinês japonês português language",
        intro: "Escolha o idioma da interface sem alterar suas gravações ou pastas.",
        steps: &[
            "No instalador, abra o seletor Idioma no cabeçalho e escolha Português (Brasil), English, Español, 简体中文 ou 日本語.",
            "No aplicativo, abra Configurações › Geral › Idioma. A interface muda imediatamente e a escolha é salva.",
            "A escolha do instalador é aplicada uma vez na abertura normal. Alterações posteriores feitas no aplicativo são preservadas ao reabrir e ao carregar perfis.",
            "Caminhos, nomes de arquivos, conteúdo do usuário, nomes reais de dispositivos e avisos legais originais permanecem intactos. Diálogos e mensagens fornecidos pelo Windows seguem o idioma do sistema.",
        ],
        tip: "As traduções são incorporadas ao aplicativo; não é necessário baixar arquivos de idioma. Chinês usa a escrita simplificada.",
        category: Some(0),
    },
    Topic {
        title: "PrintScreen personalizado",
        icon: "resolution",
        keywords: "printscreen foto tela png captura recorte atraso clipboard colar",
        intro: "Capture na resolução original, com destino e atalho próprios.",
        steps: &[
            "Abra Capturar tela em qualquer interface. Escolha monitor, janela ou região e confirme o alvo.",
            "Defina o atraso, a inclusão do cursor e a cópia da imagem para colar. O PNG sempre é salvo na pasta de capturas.",
            "Capturar agora usa esses ajustes. O atalho PrintScreen também os usa, mesmo com outro programa em foco.",
            "Em Atalhos, clique no controle e pressione a tecla ou combinação desejada. F8, F9 e F10 continuam sendo os padrões de gravação; Restaurar recupera todos os padrões.",
        ],
        tip: "Conteúdo protegido pode impedir a captura. Se PrintScreen estiver reservado pelo Windows, escolha outra combinação.",
        category: Some(4),
    },
    Topic {
        title: "Webcam e pastas do Windows",
        icon: "webcam",
        keywords: "webcam camera ampliar preview windows10 pasta videos imagens permissao armazenamento",
        intro: "Veja a câmera inteira e confira onde cada arquivo será salvo.",
        steps: &[
            "Uma câmera conectada habilita Ampliar webcam. A prévia usa toda a área disponível sem cortar a proporção; Esc fecha.",
            "A prévia da câmera é independente da opção de incluí-la na gravação. Ative Webcam para sobrepor a imagem ao vídeo.",
            "Suas pastas permite abrir ou alterar vídeos, áudio, capturas, cache, exportações e projetos. Escolher raiz prepara todas as pastas de mídia juntas.",
            "Os padrões usam Vídeos para gravações e exportações, Imagens para capturas, Música para áudio e Documentos para projetos. Somente cache e preferências ficam na pasta de dados. Use Restaurar pastas do Windows ou escolha destinos personalizados; arquivos existentes permanecem no lugar.",
        ],
        tip: "O aplicativo usa as permissões da sua conta. Pastas protegidas ou unidades desconectadas podem exigir outro destino; nenhuma proteção do Windows é desativada.",
        category: Some(5),
    },
];
pub fn contextual(page: super::app::Page) -> usize {
    use super::app::Page::*;
    match page {
        Editor => 5,
        Library => 4,
        Settings | Components => 8,
        Recovery => 11,
        Diagnostics | Capabilities => 10,
        _ => 0,
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn all_help_topics_have_steps_and_valid_destinations() {
        for t in TOPICS {
            assert!(t.steps.len() >= 4);
            assert!(!t.tip.is_empty());
            assert!(t.category.is_none_or(|i| i < 10));
        }
    }
}
