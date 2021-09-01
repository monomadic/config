unlink:
	rm $(HOME)/.config/foot
	rm $(HOME)/.config/micro
	rm $(HOME)/.config/ranger
	rm $(HOME)/.config/sway
	rm $(HOME)/.config/waybar

link:
	ln -s $(PWD)/apps/foot $(HOME)/.config/
	ln -s $(PWD)/apps/micro $(HOME)/.config/
	ln -s $(PWD)/apps/ranger $(HOME)/.config/
	ln -s $(PWD)/apps/sway $(HOME)/.config/
	ln -s $(PWD)/apps/waybar $(HOME)/.config/
