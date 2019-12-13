#? replace(sub = "\t", by = " ")
import osproc, os, strformat;

proc key_gen*(): tuple[priv: string, publ: string] =
	let priv = exec_process("openssl genrsa 2048", options = { poUsePath, poEvalCommand });
	let publ = exec_process(&"echo '{priv}' | openssl rsa -outform PEM -pubout", options = { poUsePath, poEvalCommand });
	return (priv, publ)

when is_main_module:
	let (priv, publ) = key_gen();
	echo priv;
	echo "|";
	echo publ;
