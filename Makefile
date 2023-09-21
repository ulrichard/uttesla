# https://ubports.com/de/blog/ubports-blogs-nachrichten-1/post/introduction-to-clickable-147
# https://docs.ubports.com/en/latest/appdev/platform/apparmor.html
# In desktop mode /home/phablet is mounted to ~/.clickable/home/. You can manipulate and add data there.
# If the logs target doesn't show the logs, check the following file on the device:  ~/.cache/upstart/application-click-uttesla.ulrichard_uttesla_0.0.2.log

gui: /home/richi/.clickable/home/.local/share/uttesla.ulrichard/tesla_access_token.txt
	export all_proxy=""
	export ALL_PROXY=""
	export PATH=$PATH:~/.local/bin
	clickable update || clickable desktop

test:
	export all_proxy=""
	export ALL_PROXY=""
	export PATH=$PATH:~/.local/bin
	clickable test

phone:
	export all_proxy=""
	export ALL_PROXY=""
	export PATH=$PATH:~/.local/bin
	clickable build --arch arm64
	sudo adb start-server
	sudo adb devices
	clickable install --arch arm64
	adb kill-server

publish: 
	clickable build --arch arm64
	clickable publish --apikey $(shell gpg -d api-key.gpg) --arch arm64
	clickable build --arch amd64
	clickable publish --apikey $(shell gpg -d api-key.gpg) --arch amd64
	# clickable build --arch armhf
	# clickable publish --apikey $(shell gpg -d api-key.gpg) --arch armhf

logs:
	clickable logs --arch arm64

init: 
	export PATH=$PATH:~/.local/bin
	clickable create

setup:
	sudo apt install docker.io adb git python3 python3-pip mesa-utils libgl1-mesa-glx
	pip3 install --user --break-system-packages --upgrade clickable-ut


/home/richi/.clickable/home/.local/share/uttesla.ulrichard/tesla_access_token.txt: /home/richi/.clickable/home/.local/share/uttesla.ulrichard/tesla_access_token.txt.gpg
	bash -c "gpg -d /home/richi/.clickable/home/.local/share/uttesla.ulrichard/tesla_access_token.txt.gpg > /home/richi/.clickable/home/.local/share/uttesla.ulrichard/tesla_access_token.txt"

auth: teslatte_builder
	docker run --interactive --rm --tty teslatte_ut cargo run --bin teslatte -- auth

teslatte_builder:
	docker build --tag teslatte_ut \
		--build-arg UID="$(shell id -u)" \
		.
