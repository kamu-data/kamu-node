import {Component, OnInit} from '@angular/core';
import {Observable, of, Subscription, throwError} from 'rxjs';
import {environment} from '../../../environments/environment';
import {HttpClient} from '@angular/common/http';
import {catchError, map} from 'rxjs/operators';
import {ActivatedRoute, Router} from '@angular/router';
import {AuthApi} from '../../api/auth.api';
import {CheckAuthenticated, GetAccessTokenResponse, UserInterface} from '../../interface/auth.interface';

@Component({
  selector: 'app-github-callback',
  templateUrl: './github-callback.component.html'
})

export class GithubCallbackComponent implements OnInit {

  constructor(private route: ActivatedRoute,
              private router: Router,
              private httpClient: HttpClient,
              private authApi: AuthApi) {
  }


  ngOnInit() {
      debugger;
      this.route.queryParams.subscribe(
          (param: any) => {
              debugger
              this.authApi.getUserInfoAndToken(param.code).subscribe(() => this.router.navigate(['/']));
          });
  }

}