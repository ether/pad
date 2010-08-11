import("etherpad.log");
import("sqlbase.sqlbase");
function padModelWriteToDB(args){
	log.info("************ in navigateByImageContentPlugin doing a save");
	var text = args.pad.text();
	var meta = args.pad._meta;
		
	//meta.images = [];// 'http://farm4.static.flickr.com/3067/2612399892_7df428d482.jpg';
	
	var REGEXP_IMG = /http.+\.(png|jpg)/igm;
	var matches = text.match(REGEXP_IMG);//REGEXP_IMG.exec(text);
	for(var i=0; i < matches.length;i++)  log.info("match[" + i + "]=" +matches[i]);
	
	meta.images = matches;
	
	sqlbase.putJSON("PAD_META", meta.padId, meta);
	
	//args.pad.writeToDB(); recursive loop!!
}