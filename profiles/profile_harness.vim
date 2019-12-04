let s:log_file = 'profile_log'

execute 'profile start ' . s:log_file
profile file run_profile.vim
source run_profile.vim
qall
