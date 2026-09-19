import JSZip from 'jszip';

export function sanitizeFilename(name: string): string {
  if (!name) return 'untitled';
  return name.replace(/[/\\?%*:|"<>]/g, '_').trim();
}

export interface ExportableNovelData {
  title: string;
  subtitle?: string;
  synopsis?: string;
  chapters: Array<{
    title: string;
    scenes: Array<{
      title: string;
      content: string;
    }>;
  }>;
}

/**
 * シーンごとに分割したテキストファイルを小説タイトルフォルダ内にまとめてZip出力
 */
export async function exportNovelAsSplitTxtZip(novelData: ExportableNovelData) {
  if (!novelData || !novelData.chapters) return;

  const zip = new JSZip();
  const folderName = sanitizeFilename(novelData.title || 'Zephyros_原稿');
  const folder = zip.folder(folderName) || zip;

  let sceneCounter = 1;

  novelData.chapters.forEach((ch) => {
    const chTitleClean = sanitizeFilename(ch.title);
    ch.scenes.forEach((sc) => {
      const scTitleClean = sanitizeFilename(sc.title);
      const numStr = String(sceneCounter).padStart(3, '0');
      const fileName = `${numStr}_${chTitleClean}_${scTitleClean}.txt`;

      folder.file(fileName, sc.content || '');
      sceneCounter++;
    });
  });

  const blob = await zip.generateAsync({ type: 'blob' });
  const url = URL.createObjectURL(blob);
  const a = document.createElement('a');
  a.href = url;
  a.download = `${folderName}.zip`;
  a.click();
  URL.revokeObjectURL(url);
}

/**
 * シーンごとに分割したMarkdownファイルを小説タイトルフォルダ内にまとめてZip出力
 */
export async function exportNovelAsSplitMdZip(novelData: ExportableNovelData) {
  if (!novelData || !novelData.chapters) return;

  const zip = new JSZip();
  const folderName = sanitizeFilename(novelData.title || 'Zephyros_原稿');
  const folder = zip.folder(folderName) || zip;

  let sceneCounter = 1;

  novelData.chapters.forEach((ch) => {
    const chTitleClean = sanitizeFilename(ch.title);
    ch.scenes.forEach((sc) => {
      const scTitleClean = sanitizeFilename(sc.title);
      const numStr = String(sceneCounter).padStart(3, '0');
      const fileName = `${numStr}_${chTitleClean}_${scTitleClean}.md`;

      const content = `# ${ch.title} - ${sc.title}\n\n${sc.content || ''}\n`;
      folder.file(fileName, content);
      sceneCounter++;
    });
  });

  const blob = await zip.generateAsync({ type: 'blob' });
  const url = URL.createObjectURL(blob);
  const a = document.createElement('a');
  a.href = url;
  a.download = `${folderName}_Markdown.zip`;
  a.click();
  URL.revokeObjectURL(url);
}

function escapeXml(str: string): string {
  if (!str) return '';
  return str
    .replace(/&/g, '&amp;')
    .replace(/</g, '&lt;')
    .replace(/>/g, '&gt;')
    .replace(/"/g, '&quot;')
    .replace(/'/g, '&apos;');
}

function convertTextToXhtmlParagraphs(text: string): string {
  if (!text) return '<p></p>';
  const lines = text.split(/\r?\n/);
  const htmlLines = lines.map((line) => {
    const trimmed = line.trim();
    if (!trimmed) {
      return '<p class="empty-line"><br/></p>';
    }

    const escaped = escapeXml(trimmed);
    const rubyConverted = escaped.replace(
      /(?:[｜|]([^\s《》\n]+)|([\u4E00-\u9FFF\u3005\u3007\u30A0-\u30FFa-zA-Z0-9]+))《([^》]+)》/g,
      (_match, p1, p2, p3) => {
        const base = p1 || p2;
        return `<ruby>${base}<rt>${p3}</rt></ruby>`;
      }
    );

    const isDialogue = trimmed.startsWith('「') || trimmed.startsWith('『');
    const pClass = isDialogue ? ' class="dialogue"' : '';
    return `<p${pClass}>${rubyConverted}</p>`;
  });

  return htmlLines.join('\n');
}

/**
 * 小説全データを電子書籍標準フォーマット (EPUB 3 / EPUB 2 両対応) の .epub ファイルとして出力
 */
export async function exportNovelAsEpub(novelData: ExportableNovelData) {
  if (!novelData || !novelData.chapters) return;

  const zip = new JSZip();

  // 1. mimetype (非圧縮 STORE 必須)
  zip.file('mimetype', 'application/epub+zip', { compression: 'STORE' });

  // 2. META-INF/container.xml
  zip.file(
    'META-INF/container.xml',
    `<?xml version="1.0" encoding="UTF-8"?>
<container version="1.0" xmlns="urn:oasis:names:tc:opendocument:xmlns:container">
  <rootfiles>
    <rootfile full-path="OEBPS/content.opf" media-type="application/oebps-package+xml"/>
  </rootfiles>
</container>`
  );

  // 3. OEBPS/style.css
  const styleCss = `@charset "UTF-8";
body {
  font-family: "Hiragino Mincho ProN", "Yu Mincho", "游明朝", "MS Mincho", serif;
  line-height: 1.8;
  margin: 5%;
  text-align: justify;
  color: #111827;
}
h1 {
  font-size: 1.6em;
  text-align: center;
  margin-top: 2em;
  margin-bottom: 1.2em;
  font-weight: bold;
  border-bottom: 1px solid #e5e7eb;
  padding-bottom: 0.5em;
}
h2 {
  font-size: 1.2em;
  text-align: center;
  margin-top: 1em;
  margin-bottom: 1em;
  font-weight: normal;
  color: #4b5563;
}
.synopsis {
  margin: 2em 10%;
  padding: 1em;
  background-color: #f9fafb;
  border-left: 4px solid #6366f1;
  font-size: 0.95em;
  line-height: 1.7;
}
h3 {
  font-size: 1.25em;
  margin-top: 2em;
  margin-bottom: 1em;
  border-bottom: 1px dashed #cccccc;
  padding-bottom: 0.3em;
}
p {
  text-indent: 1em;
  margin-top: 0;
  margin-bottom: 0;
}
p.dialogue {
  text-indent: 0;
}
p.empty-line {
  text-indent: 0;
  margin-top: 1em;
}
ruby rt {
  font-size: 0.55em;
}`;

  zip.file('OEBPS/style.css', styleCss);

  const uuid = `zephyros-${Date.now()}-${Math.random().toString(36).substr(2, 6)}`;
  const titleClean = novelData.title || '無題の物語';
  const escapedTitle = escapeXml(titleClean);
  const escapedSubtitle = escapeXml(novelData.subtitle || '');
  const escapedSynopsis = escapeXml(novelData.synopsis || '');

  // 4. OEBPS/text/title_page.xhtml
  const titlePageXhtml = `<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE html>
<html xmlns="http://www.w3.org/1999/xhtml" xmlns:epub="http://www.idpf.org/2007/ops" xml:lang="ja" lang="ja">
<head>
  <meta charset="UTF-8"/>
  <title>${escapedTitle}</title>
  <link rel="stylesheet" type="text/css" href="../style.css"/>
</head>
<body>
  <h1>${escapedTitle}</h1>
  ${escapedSubtitle ? `<h2>${escapedSubtitle}</h2>` : ''}
  ${escapedSynopsis ? `<div class="synopsis"><p>${escapedSynopsis.replace(/\n/g, '<br/>')}</p></div>` : ''}
</body>
</html>`;

  zip.file('OEBPS/text/title_page.xhtml', titlePageXhtml);

  // 5. 各話 XHTML ファイル作成
  const manifestItems: string[] = [
    `<item id="nav" href="nav.xhtml" media-type="application/xhtml+xml" properties="nav"/>`,
    `<item id="ncx" href="toc.ncx" media-type="application/x-dtbncx+xml"/>`,
    `<item id="style" href="style.css" media-type="text/css"/>`,
    `<item id="title_page" href="text/title_page.xhtml" media-type="application/xhtml+xml"/>`,
  ];
  const spineItems: string[] = [`<itemref idref="title_page"/>`];
  const navPointNcx: string[] = [
    `<navPoint id="nav-title" playOrder="1">
      <navLabel><text>表紙・作品概要</text></navLabel>
      <content src="text/title_page.xhtml"/>
    </navPoint>`,
  ];
  const navListHtml: string[] = [
    `<li><a href="text/title_page.xhtml">表紙・作品概要</a></li>`,
  ];

  let playOrder = 2;

  novelData.chapters.forEach((ch, cIdx) => {
    const chNumStr = String(cIdx + 1).padStart(3, '0');
    const chId = `chapter_${chNumStr}`;
    const chFileName = `${chId}.xhtml`;
    const escapedChTitle = escapeXml(ch.title || `第${cIdx + 1}話`);

    // 各シーンの本文を統合
    const scenesBody = ch.scenes
      .map((sc, sIdx) => {
        const scTitleEscaped = escapeXml(sc.title || `シーン${sIdx + 1}`);
        const scContentXhtml = convertTextToXhtmlParagraphs(sc.content || '');
        return `<section class="scene">
          <h3>${scTitleEscaped}</h3>
          ${scContentXhtml}
        </section>`;
      })
      .join('\n');

    const chapterXhtml = `<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE html>
<html xmlns="http://www.w3.org/1999/xhtml" xmlns:epub="http://www.idpf.org/2007/ops" xml:lang="ja" lang="ja">
<head>
  <meta charset="UTF-8"/>
  <title>${escapedChTitle}</title>
  <link rel="stylesheet" type="text/css" href="../style.css"/>
</head>
<body>
  <h1>${escapedChTitle}</h1>
  ${scenesBody}
</body>
</html>`;

    zip.file(`OEBPS/text/${chFileName}`, chapterXhtml);

    manifestItems.push(`<item id="${chId}" href="text/${chFileName}" media-type="application/xhtml+xml"/>`);
    spineItems.push(`<itemref idref="${chId}"/>`);
    navPointNcx.push(`<navPoint id="nav-${chId}" playOrder="${playOrder}">
      <navLabel><text>${escapedChTitle}</text></navLabel>
      <content src="text/${chFileName}"/>
    </navPoint>`);
    navListHtml.push(`<li><a href="text/${chFileName}">${escapedChTitle}</a></li>`);

    playOrder++;
  });

  // 6. OEBPS/nav.xhtml (EPUB 3 Toc)
  const navXhtml = `<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE html>
<html xmlns="http://www.w3.org/1999/xhtml" xmlns:epub="http://www.idpf.org/2007/ops" xml:lang="ja" lang="ja">
<head>
  <meta charset="UTF-8"/>
  <title>目次</title>
  <link rel="stylesheet" type="text/css" href="style.css"/>
</head>
<body>
  <nav epub:type="toc" id="toc">
    <h1>目次</h1>
    <ol>
      ${navListHtml.join('\n      ')}
    </ol>
  </nav>
</body>
</html>`;

  zip.file('OEBPS/nav.xhtml', navXhtml);

  // 7. OEBPS/toc.ncx (EPUB 2 Toc)
  const tocNcx = `<?xml version="1.0" encoding="UTF-8"?>
<ncx xmlns="http://www.daisy.org/z3986/2005/ncx/" version="2005-1">
  <head>
    <meta name="dtb:uid" content="urn:uuid:${uuid}"/>
    <meta name="dtb:depth" content="1"/>
    <meta name="dtb:totalPageCount" content="0"/>
    <meta name="dtb:maxPageNumber" content="0"/>
  </head>
  <docTitle><text>${escapedTitle}</text></docTitle>
  <navMap>
    ${navPointNcx.join('\n    ')}
  </navMap>
</ncx>`;

  zip.file('OEBPS/toc.ncx', tocNcx);

  // 8. OEBPS/content.opf (Package Manifest & Spine)
  const isoDate = new Date().toISOString().split('.')[0] + 'Z';
  const contentOpf = `<?xml version="1.0" encoding="UTF-8"?>
<package xmlns="http://www.idpf.org/2007/opf" unique-identifier="pub-id" version="3.0">
  <metadata xmlns:dc="http://purl.org/dc/elements/1.1/">
    <dc:identifier id="pub-id">urn:uuid:${uuid}</dc:identifier>
    <dc:title>${escapedTitle}</dc:title>
    <dc:language>ja</dc:language>
    <dc:creator>Zephyros Novel Generator</dc:creator>
    <meta property="dcterms:modified">${isoDate}</meta>
  </metadata>
  <manifest>
    ${manifestItems.join('\n    ')}
  </manifest>
  <spine toc="ncx">
    ${spineItems.join('\n    ')}
  </spine>
</package>`;

  zip.file('OEBPS/content.opf', contentOpf);

  // EPUB Blob 生成 ＆ ダウンロードトリガー
  const epubBlob = await zip.generateAsync({
    type: 'blob',
    mimeType: 'application/epub+zip',
    compression: 'DEFLATE',
    compressionOptions: { level: 9 },
  });

  const folderName = sanitizeFilename(titleClean);
  const url = URL.createObjectURL(epubBlob);
  const a = document.createElement('a');
  a.href = url;
  a.download = `${folderName}.epub`;
  a.click();
  URL.revokeObjectURL(url);
}
