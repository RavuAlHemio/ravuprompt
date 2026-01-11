# remember more
HISTSIZE=1000000000
SAVEHIST="$HISTSIZE"

# remember it between shell calls
HISTFILE="$HOME/.zsh_history"

# keep appending into the history file
setopt INC_APPEND_HISTORY

# store timestamps with history
setopt EXTENDED_HISTORY

# load completion logic
autoload -U compinit; compinit

# modules to enable up and down arrow to search through history
autoload -U up-line-or-beginning-search
autoload -U down-line-or-beginning-search
zle -N up-line-or-beginning-search
zle -N down-line-or-beginning-search

# keybindings
for md in emacs viins vicmd
do
	# up arrow: search backward in history
	bindkey -M "$md" "^[[A" up-line-or-beginning-search
	[ -n "${terminfo[kcuu1]}" ] && bindkey -M "$md" "${terminfo[kcuu1]}" up-line-or-beginning-search

	# down arrow: search backward in history
	bindkey -M "$md" "^[[B" down-line-or-beginning-search
	[ -n "${terminfo[kcud1]}" ] && bindkey -M "$md" "${terminfo[kcud1]}" down-line-or-beginning-search

	# ctrl + left arrow: back one word
	bindkey -M "$md" '^[[1;5D' backward-word

	# ctrl + right arrow: forward one word
	bindkey -M "$md" '^[[1;5C' forward-word

	if [ -n "${terminfo[kdch1]}" ]
	then
		bindkey -M "$md" "${terminfo[kdch1]}" delete-char
	else
		# two common delete characters
		bindkey -M "$md" "^[[3~" delete-char
		bindkey -M "$md" "^[3;5~" delete-char
	fi
done

# Ctrl-W deletes less
WORDCHARS=""

# nicer prompt
setopt promptsubst
PROMPT='$("$HOME/bin/ravuprompt")'

# terminal title magic
function changetitles {
	setopt localoptions
	setopt nopromptsubst

	tabname="$1"
	if [ $# -gt 1 ]
	then
		winname="$2"
	else
		winname="$tabname"
	fi

	case "$TERM" in
		cygwin|xterm*|putty*|rxvt*|konsole*|ansi|mlterm*|alacritty*|st*|foot*|contour*|wezterm*)
			print -Pn "\e]2;${winname:q}\a"
			print -Pn "\e]1;${tabname:q}\a"
			;;
		screen*|tmux*)
			# set "hardstatus"
			print -Pn "\ek${tabname:q}\e\\"
			;;
		*)
			case "$TERM_PROGRAM" in
				iTerm.app)
					print -Pn "\e]2;${winname:q}\a"
					print -Pn "\e]1;${tabname:q}\a"
					;;
				*)
					# terminfo to the rescue?
					if [ -n "${terminfo[tsl]}" -a -n "${terminfo[fsl]}" ]
					then
						print -Pn "${terminfo[tsl]}${tabname}${terminfo[fsl]}"
					fi
					;;
			esac
			;;
	esac
}
function ravuprompt_title_precmd {
	changetitles "%n@%m:%~"
}
function ravuprompt_title_preexec {
	changetitles "$1"
}
autoload -U add-zsh-hook
add-zsh-hook precmd ravuprompt_title_precmd
add-zsh-hook preexec ravuprompt_title_preexec

# life is a coloring book
alias ls='/bin/ls --color=auto'
alias grep='/bin/grep --color=auto'
export LSCOLORS="Gxfxcxdxbxegedabagacad"
export LS_COLORS='no=00:fi=00:di=01;34:ln=00;36:pi=40;33:so=01;35:do=01;35:bd=40;33;01:cd=40;33;01:or=41;33;01:ex=00;32:*.cmd=00;32:*.exe=01;32:*.com=01;32:*.bat=01;32:*.btm=01;32:*.dll=01;32:*.tar=00;31:*.tbz=00;31:*.tgz=00;31:*.rpm=00;31:*.deb=00;31:*.arj=00;31:*.taz=00;31:*.lzh=00;31:*.lzma=00;31:*.zip=00;31:*.zoo=00;31:*.z=00;31:*.Z=00;31:*.gz=00;31:*.bz2=00;31:*.tb2=00;31:*.tz2=00;31:*.tbz2=00;31:*.avi=01;35:*.bmp=01;35:*.fli=01;35:*.gif=01;35:*.jpg=01;35:*.jpeg=01;35:*.mng=01;35:*.mov=01;35:*.mpg=01;35:*.pcx=01;35:*.pbm=01;35:*.pgm=01;35:*.png=01;35:*.ppm=01;35:*.tga=01;35:*.tif=01;35:*.xbm=01;35:*.xpm=01;35:*.dl=01;35:*.gl=01;35:*.wmv=01;35:*.aiff=00;32:*.au=00;32:*.flac=00;32:*.mid=00;32:*.mp3=00;32:*.ogg=00;32:*.voc=00;32:*.wav=00;32:'

# don't cycle through completions with tab
setopt noautomenu

if [ -r ~/.exports ]
then
    . ~/.exports
fi

if [ -r ~/.aliases ]
then
    . ~/.aliases
fi

# many colors!
if [ "z$TERM" = "zxterm" -a "x$VTE_VERSION" != "x" ]
then
    TERM=xterm-256color
fi
