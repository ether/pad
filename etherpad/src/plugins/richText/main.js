import("etherpad.log");
import("plugins.richText.hooks");
import("plugins.richText.importexport");
import("plugins.richText.static.js.main");

function richTextInit() {
    this.hooks = ['editBarItemsLeftPad', 'editBarItemsLeftPadView', 'aceAttribsToClasses',
                  'aceCreateDomLine', 'aceCreateStructDomLine', 'exportInlineStyle', 'exportLineMarkerStyle',
                  'collectContentPre', 'collectContentPost', 'afterExport'];
    this.description = 'Rich Text editing in etherpad';
    this.client = new main.richTextInit();
    this.editBarItemsLeftPad = hooks.editBarItemsLeftPad;
    this.editBarItemsLeftPadView = hooks.editBarItemsLeftPadView;
    this.aceAttribsToClasses = main.aceAttribsToClasses;
    this.aceCreateDomLine = main.aceCreateDomLine;
    this.aceCreateStructDomLine = main.aceCreateStructDomLine; 
    this.exportInlineStyle = importexport.exportInlineStyle;
    this.exportLineMarkerStyle = importexport.exportLineMarkerStyle;
    this.collectContentPre = importexport.collectContentPre;
    this.collectContentPost = importexport.collectContentPost;
    this.afterExport = importexport.afterExport;
    this.install = install;
    this.uninstall = uninstall;
}

function install() {
    log.info("Installing richText");
}

function uninstall() {
    log.info("Uninstalling richText");
}

