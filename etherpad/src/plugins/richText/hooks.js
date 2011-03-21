import("etherpad.log");
import("faststatic");
import("etherpad.utils.*");
import("etherpad.globals.*");
import("dispatch.{Dispatcher,PrefixMatcher,forward}");
import("etherpad.helpers");


function initRunTime(){
	helpers.addToHead('<script src="http://ajax.googleapis.com/ajax/libs/dojo/1.6/dojo/dojo.xd.js" djConfig="parseOnLoad: true"></script>');
	helpers.addToHead('<script src="/static/js/plugins/richText/richtextui.js"></script>');
	helpers.addToHead('<script src="/static/js/plugins/richText/editor.js"></script>');
	helpers.addToHead('<link rel="stylesheet" type="text/css" href="http://ajax.googleapis.com/ajax/libs/dojo/1.6/dijit/themes/claro/claro.css" />');
	helpers.addToHead('<link rel="stylesheet" type="text/css" href="/static/css/plugins/richText/claropatch.css" />');
	helpers.addBodyClass("claro");
	helpers.addBodyClass("maximized");  //remove full window toggle button ?
}


function editBarItemsLeftPad(arg) {
  initRunTime();
  return arg.template.include('richTextEditbarButtons.ejs', undefined, ['richText']);
}
