# Ata de Aplicação de Prova — Especificação

Nov 8, 2026 · @Vinicius

## Objetivo

A ata é o registro oficial, impresso e preenchido à mão, de uma aplicação de prova presencial na UFRGS. Ela documenta quem estava presente, quando a prova começou e terminou, quem saiu da sala e o que aconteceu de anormal, e é assinada pelo(s) fiscal(is) ao final.

- **Quem usa:** o aplicador/fiscal durante a prova (preenche a caneta) e os alunos (assinam na devolução).
- **Quem consome depois:** o professor responsável e a secretaria/departamento, como evidência em caso de recurso, fraude, segunda chamada ou contestação de presença.
- **O que o software faz:** pré-preenche tudo que é conhecido antes da prova (disciplina, turma, horários, lista de alunos) e deixa campos em branco, bem delimitados, para o que só se sabe na hora (presença, horários reais, saídas, ocorrências, totais, assinaturas).
- **Saída:** um PDF A4 pronto para impressão em preto e branco, em português do Brasil.

## Informações coletadas

Os dados se dividem em quatro grupos: identificação da aplicação, horários, lista de alunos e opções extras. Todos são conhecidos antes da prova; nada do que é preenchido à mão é armazenado.

### Identificação da aplicação

| Campo | Tipo | Obrigatório | Observações |
| --- | --- | --- | --- |
| Atividade de Ensino (disciplina) | texto | sim | Ex.: "Introdução à Programação" |
| Período Letivo | texto | sim | Padrão sugerido: `ANO/1` antes de agosto, `ANO/2` a partir de agosto |
| Professor Responsável | texto | sim |  |
| Turma(s) | texto | sim | Pode listar várias ("A, B") |
| Avaliação | texto | sim | Ex.: "1ª Avaliação" |
| Tipo de aplicação | escolha | sim | Regular, Substitutiva ou Especial |
| Data | data | sim | Exibida como dd/mm/aaaa |
| Sala | texto | sim | Composta como "\<sala> - Prédio \<prédio>" |
| Linhas de saídas temporárias | inteiro | sim | Padrão 4; quantas linhas em branco imprimir |

### Horários

| Campo | Tipo | Observações |
| --- | --- | --- |
| Início previsto | hora (HH:MM) |  |
| Duração | minutos |  |
| Término previsto | hora (HH:MM) | Sugerido como início + duração, mas editável (pode haver tolerância) |

### Alunos

| Campo | Tipo | Observações |
| --- | --- | --- |
| Nº | inteiro | Ordem na lista, renumerado automaticamente após inclusão/remoção |
| Matrícula (ID) | inteiro | Chave do aluno; duplicatas geram aviso |
| Nome | texto |  |
| Turma | texto |  |
| E-mail | texto | Armazenado, não impresso |
| Observações | texto opcional | Impressa sob o nome ("Obs.: …"), ex.: atendimento especial |

A lista pode ser importada de CSV com colunas `ID`, `nome`, `turma`, `email` (formato de exportação do sistema acadêmico). Reimportar mescla por matrícula e preserva observações já registradas.

### Opções extras (todas desligadas por padrão)

| Opção | Dado adicional | Efeito na ata |
| --- | --- | --- |
| Coluna de turma | — | Adiciona coluna "Turma" na lista de presença |
| Hora de entrega individual | — | Adiciona coluna "Entrega" na lista de presença |
| Segundo fiscal | — | Duas assinaturas (Titular e Auxiliar) em vez de "Aplicador" |
| Código da disciplina | texto | Linha "Código" na identificação |
| Instituto/Departamento | texto | Linha na identificação |
| Materiais permitidos / regras | texto | Linha na identificação |
| Local e data por extenso | cidade | "Porto Alegre, 27 de agosto de 2026." antes das assinaturas |

## Formato e estrutura do documento

A ata é um PDF A4, margens de 1,8 cm, fonte sem serifa (Atkinson Hyperlegible) em 11 pt, pensado para impressão P&B e preenchimento a caneta. As seções aparecem nesta ordem:

1. **Cabeçalho e rodapé (todas as páginas)**
   - Cabeçalho: "ATA DE APLICAÇÃO DE PROVA" à esquerda, "UFRGS" à direita.
   - Rodapé: "Gerado em dd/mm/aaaa HH:MM" e "Folha X de Y".
2. **Título** "Aplicação de Prova", centralizado.
3. **Identificação** — tabela rótulo/valor: Atividade de Ensino, \[Código\], Período Letivo, Professor Responsável, \[Instituto/Departamento\], Turma(s), Avaliação + Aplicação (mesma linha), Data, Horário Previsto ("18:30 — 20:10 (100 minutos)"), Sala, \[Materiais Permitidos\]. Colchetes = opcional.
4. **Registro de Presença**
   - Caixa de instruções: marcar **P** (presente) ou **A** (ausente) no início; **T** (tarde) para quem chega depois; assinatura no ato da devolução.
   - Tabela que quebra entre páginas, repetindo o cabeçalho com "(continuação da lista de presença)" e "continua na próxima página…".
   - Colunas: Nº · Matrícula · Nome (+ observação em itálico) · \[Turma\] · Presença (caixa de 1 caractere) · Assinatura (linha pontilhada) · \[Entrega\].
   - O nome é a única coluna em tamanho normal; as demais usam fonte menor. Larguras são ajustadas para cada combinação de colunas opcionais.
5. **Horários** — lado a lado, o previsto (impresso em destaque) e o real (caixas HH:MM em branco):
   - Início previsto → Início real
   - Tempo de prova (minutos)
   - Limite previsto → Limite calculado (início real + duração)
   - Término real
6. **Registro de Saídas Temporárias** — N linhas em branco (configurável) com: Nº do aluno, hora de saída, hora de retorno, rubrica do fiscal.
7. **Ocorrências** — instrução (fraude, mal-estar, atraso, saída antecipada etc., com horário e aluno) e 5 linhas numeradas.
8. **Encerramento** — nunca dividido entre páginas.
   - Total de matriculados na lista (impresso).
   - Caixas para: presentes, ausentes, provas recolhidas, folhas extras distribuídas e recolhidas.
   - Conferência final (caixas de marcar): provas recolhidas conferem com presentes; todas as folhas extras recolhidas; não houve irregularidades; houve irregularidades registradas.
9. **Local e data** (opcional) — "Cidade, DD de mês de AAAA."
10. **Assinaturas** — "Aplicador" com nome legível; ou, com segundo fiscal, "Fiscal Titular" e "Fiscal Auxiliar" lado a lado.

Campos de preenchimento manual são padronizados: caixas de dígito para horas (`__:__`), datas e contagens, e linhas pontilhadas para texto livre e assinaturas.

## Requisitos

### Funcionais

1. Coletar os dados de identificação, horários, alunos e opções extras descritos acima.
2. Gerenciar a lista de alunos: importar CSV, adicionar, remover, editar, listar; renumerar automaticamente; avisar sobre matrículas duplicadas.
3. Salvar uma aplicação em JSON e reabri-la depois para editar (alunos, linhas de saída, opções) e regerar a ata. Arquivos antigos sem campos novos devem continuar abrindo (campos ausentes assumem o padrão).
4. Gerar o PDF da ata e abri-lo no visualizador do sistema.
5. Formatar datas e horas no padrão brasileiro (dd/mm/aaaa, HH:MM, mês por extenso em minúsculas).
6. Mostrar ou ocultar linhas, colunas e assinaturas conforme as opções extras, mantendo a tabela de presença dentro da largura da página.

### Não funcionais

- **Segurança do texto livre:** qualquer valor digitado (nomes, observações, disciplina) deve ser escapado para o formato de saída. No LaTeX atual, os caracteres `\ { } _ ^ % & # $ ~` são escapados; "Maria & Souza\_1" precisa aparecer literalmente.
- **Geração autocontida:** o PDF é compilado dentro do processo (Tectonic), sem exigir LaTeX instalado.
- **Legibilidade impressa:** P&B, linhas em cinza suave, nome do aluno em destaque, cabeçalho da tabela repetido a cada página, bloco de encerramento nunca partido.
- **Privacidade:** a ata, o `.tex` intermediário e os JSONs contêm dados pessoais (nome, matrícula, e-mail) e não devem ir para controle de versão nem ser expostos sem necessidade. O e-mail não é impresso.
- **Idioma:** documento e interface em português do Brasil.
- **Rastreabilidade:** cada ata traz data/hora de geração e paginação "Folha X de Y".

## Funcionalidades auxiliares e pontos para a nova aplicação

Além da ata, a ferramenta atual exporta o cronograma da prova para o [OnTime](https://ontime.vini84200.cc/mcp) (cronômetro/rundown exibido na sala), a partir dos mesmos dados.

| Funcionalidade | Entrada | Saída |
| --- | --- | --- |
| Persistência da aplicação | Dados coletados | `application.json`, reabrível e editável |
| Importação de alunos | CSV `ID,nome,turma,email` | Lista de presença, mesclada por matrícula |
| Cronograma OnTime (planilha) | Sala, disciplina, horários | `.xlsx` importável em Rundown › Import from spreadsheet |
| Cronograma OnTime (JSON) | Idem | Entradas para a ferramenta MCP `ontime_batch_create_entries` |

O cronograma tem 3 entradas: **ABERTURA** (marco: abertura da sala), **PROVA** (evento cronometrado em contagem regressiva, com aviso 10 min e alerta 2 min antes do fim, reduzidos para provas curtas) e **ENCERRAMENTO** (marco: conferência e recolhimento).

### Limitações atuais que a nova aplicação pode resolver

- Os registros feitos à mão (presença, horários reais, saídas, ocorrências, totais) não voltam para o sistema; não há versão digital da ata preenchida.
- "Horário limite calculado" e totais de presentes/ausentes são contas manuais que um registro digital faria sozinho.
- Instituição ("UFRGS"), número de linhas de ocorrências (5) e textos de instrução estão fixos no modelo.
- Uma ata por aplicação: não há várias salas para a mesma prova nem divisão da turma entre salas.
- Interface só de terminal, sem uso por vários fiscais ao mesmo tempo.

### Perguntas em aberto

- A nova aplicação deve manter o PDF impresso como documento oficial, ou a ata digital assinada passa a valer?
- Quais outros documentos além da ata (lista de chamada, etiquetas, relatório de notas) entram no escopo?
