#!/usr/bin/env perl

use strict;
use warnings;
use Time::HiRes qw(time);

# Flush standard output buffer immediately
$| = 1;

# ANSI color constants
my $YELLOW = "\e[1;33m";
my $RESET  = "\e[0m";
my $RED = "\e[4;31m";

# Data structure associating IDs with their text and execution commands
my @menu_options = 
(
    { id => '1', text => 'Windows 10+ Release', cmd => 'cargo build -r' },
    { id => '2', text => 'Windows 10+ Debug',   cmd => 'cargo build' },
    { id => '3', text => 'Linux Release',       cmd => 'cargo linux --release' },
    { id => '4', text => 'Linux Debug',         cmd => 'cargo linux' },
    { id => '5', text => 'Windows 7 Release',   cmd => 'cargo win7r' },
    { id => '6', text => 'Windows 7 Debug',     cmd => 'cargo win7d' },
    { id => '-', text => '-' },
    { 
        id    => '7', 
        text  => 'Build All Debug', 
        steps => 
        [
            { text => 'Windows 10+ Debug', cmd => 'cargo build' },
            { text => 'Linux Debug',       cmd => 'cargo linux' },
            { text => 'Windows 7 Debug',   cmd => 'cargo win7d' }
        ] 
    },
    { 
        id    => '8', 
        text  => 'Build All Release', 
        steps => 
        [
            { text => 'Windows 10+ Release', cmd => 'cargo build --release' },
            { text => 'Linux Release',       cmd => 'cargo linux --release' },
            { text => 'Windows 7 Release',   cmd => 'cargo win7r' }
        ] 
    }
);

# Build a fast lookup hash for the input loop
my %menu_map;
foreach my $option (@menu_options)
{
    if ($option->{id} ne '-')
    {
        $menu_map{$option->{id}} = $option;
    }
}

sub execute_build
{
    my ($text, $cmd) = @_;
    
    print "\n${YELLOW}[+] Executing: $text ($cmd)${RESET}\n";
    print "----------------------------------------\n";
    
    my $start_time = time();
    my $exit_status = system($cmd);
    my $end_time = time();
    
    print "----------------------------------------\n";
    
    my $duration = sprintf("%.2f", $end_time - $start_time);
    my $real_code;
    
    if ($exit_status == -1)
    {
        print "[-] Execution failed: $!\n";
        $real_code = 1;
    }
    else
    {
        $real_code = $exit_status >> 8;
        
        if ($real_code == 0)
        {
            print "${YELLOW}[+] Build completed successfully in ${RESET}$duration ${YELLOW}seconds.${RESET}\n";
        }
        else
        {
            print "${RED}[-] Build failed with exit code $real_code (Duration: $duration seconds).${RESET}\n";
        }
    }
    
    return $real_code;
}

while (1)
{
    print "\n";
    print "========================================\n";
    print " RIVER Build Menu\n";
    print "========================================\n";
    
    # Dynamically generate the menu from the data structure
    foreach my $option (@menu_options)
    {
        if ($option->{id} eq '-')
        {
            print " -\n";
        }
        else
        {
            print " $option->{id}.   $option->{text}\n";
        }
    }
    
    print " Q/E. Quit/Exit\n";
    print "========================================\n";
    print "Select an option: ";
    
    my $input = <STDIN>;
    
    # Handle EOF (e.g., Ctrl+D or piped input ending)
    if (!defined $input)
    {
        print "\nExiting...\n";
        exit(0);
    }
    
    chomp($input);
    
    # Trim leading and trailing whitespace
    $input =~ s/^\s+|\s+$//g;
    $input = lc($input);
    
    if (exists $menu_map{$input})
    {
        my $selection = $menu_map{$input};
        
        if (exists $selection->{cmd})
        {
            # Single command execution
            exit(execute_build($selection->{text}, $selection->{cmd}));
        }
        elsif (exists $selection->{steps})
        {
            # Chained 'Build All' execution
            foreach my $step (@{$selection->{steps}})
            {
                my $code = execute_build($step->{text}, $step->{cmd});
                
                if ($code != 0)
                {
                    print "\n[-] $selection->{text} aborted due to failure in: $step->{text} ($step->{cmd})\n";
                    exit($code);
                }
            }
            
            print "\n[+] $selection->{text} completed successfully!\n";
            exit(0);
        }
    }
    elsif ($input eq 'q' || $input eq 'e')
    {
        exit(0);
    }
    else
    {
        print "\n[!] Invalid input: '$input'. Please select a valid option from the menu.\n";
    }
}