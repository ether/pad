function chatNotificationPluginInit() {
 this.hooks = ['chatNotification'];
 this.chatNotification = chatNotification;
}

//	This function plays sound
//	It relays on flash player wavplayer.swf
//	This player is relesed under GNU GPL v3 licence
//	https://github.com/francois2metz/WavPlayer

function chatNotification() {

  function getPlayer(pid) {
	var obj = document.getElementById(pid);
		if (obj.doPlay) return obj;
		for(i=0; i<obj.childNodes.length; i++) {
			var child = obj.childNodes[i];
			if (child.tagName == "EMBED") return child;
		}
	}
	function doPlay(fname) {
		var player=getPlayer("audio1");
		player.play(fname);
	}
	function doStop() {
		var player=getPlayer("audio1");
		player.doStop();
	}
	doPlay('/static/html/plugins/chatNotification/audio/snd.wav');
}

/* used on the client side only */
chatNotification = new chatNotificationPluginInit();
