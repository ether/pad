$(function(){
	$('#new select').change(function(){
		var $this = $(this);
		var path = $this.val().replace(/\/$/,'') + "/";
		$this.val($this.find("option:first").val() );
		var result = prompt('Enter name for new pad in section ' + path );
		result = result.replace(/\s/gi, '-').toLowerCase();
		if (result) {
			window.location.href = path + result + "/+edit";
		}
	});
})
